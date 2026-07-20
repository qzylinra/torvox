package main

import (
	"bytes"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"flag"
	"io"
	"log"
	"net/http"
	"os"
	"strings"
	"sync"
	"time"
)

const (
	userAgent     = "opencode/latest/1.3.15/cli"
	clientHeader  = "cli"
	sessionHexLen = 26
	serverTimeout = 5 * time.Minute
	idleTimeout   = 2 * time.Minute
	cacheTTL      = 5 * time.Minute
)

type config struct {
	listen    string
	upstream  string
	modelsDev string
	quiet     bool
}

// costCache holds models.dev cost info with a short TTL so we do not refetch the
// (large) catalog on every request, and so an outage degrades gracefully.
type costCache struct {
	mu      sync.Mutex
	entries map[string]costEntry
	expires time.Time
	ok      bool
}

func (c *costCache) get() (map[string]costEntry, bool) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.ok && time.Now().Before(c.expires) {
		return c.entries, true
	}
	return nil, false
}

func (c *costCache) set(entries map[string]costEntry) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.entries = entries
	c.expires = time.Now().Add(cacheTTL)
	c.ok = true
}

type modelListResponse struct {
	Object string      `json:"object"`
	Data   []modelInfo `json:"data"`
}

type modelInfo struct {
	ID      string `json:"id"`
	Object  string `json:"object"`
	OwnedBy string `json:"owned_by"`
}

type upstreamModelList struct {
	Data []struct {
		ID string `json:"id"`
	} `json:"data"`
}

type modelsDevResponse struct {
	OpenCode struct {
		Models map[string]struct {
			Status string `json:"status"`
			Cost   struct {
				Input float64 `json:"input"`
			} `json:"cost"`
		} `json:"models"`
	} `json:"opencode"`
}

type proxy struct {
	config config
	client *http.Client
	logger *log.Logger
	cost   costCache
}

func main() {
	var cfg config
	flag.StringVar(&cfg.listen, "listen", "127.0.0.1:8787", "bind address")
	flag.StringVar(&cfg.upstream, "upstream", "https://opencode.ai/zen/v1", "upstream gateway base (includes /v1)")
	flag.StringVar(&cfg.modelsDev, "modelsdev", "https://models.dev/api.json", "models.dev api url for cost info")
	flag.BoolVar(&cfg.quiet, "quiet", false, "reduce logging")
	flag.Parse()

	logger := log.New(os.Stderr, "", log.LstdFlags)
	if cfg.quiet {
		logger.SetOutput(io.Discard)
	}

	p := &proxy{
		config: cfg,
		client: &http.Client{Timeout: serverTimeout},
		logger: logger,
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/", p.handle)

	server := &http.Server{
		Addr:         cfg.listen,
		Handler:      mux,
		ReadTimeout:  serverTimeout,
		WriteTimeout: serverTimeout,
		IdleTimeout:  idleTimeout,
	}

	log.New(os.Stderr, "", log.LstdFlags).Printf("openclaude-zen-free listening on %s -> %s", cfg.listen, cfg.upstream)
	if err := server.ListenAndServe(); err != nil {
		log.Fatalf("server error: %v", err)
	}
}

func normalizePath(path string) string {
	if path == "/v1" {
		return "/"
	}
	if strings.HasPrefix(path, "/v1/") {
		return strings.TrimPrefix(path, "/v1")
	}
	return path
}

func (p *proxy) handle(w http.ResponseWriter, r *http.Request) {
	normalized := normalizePath(r.URL.Path)

	switch {
	case r.Method == http.MethodGet && normalized == "/models":
		p.handleModels(w, r, normalized)
	case r.Method == http.MethodPost:
		p.handlePost(w, r, normalized)
	default:
		p.forward(w, r, normalized, nil, "")
	}
}

func (p *proxy) handleModels(w http.ResponseWriter, r *http.Request, normalized string) {
	freeIDs, err := p.freeModels()
	if err != nil {
		p.logger.Printf("path=%s error computing free models: %v", r.URL.Path, err)
		http.Error(w, "failed to compute free models: "+err.Error(), http.StatusBadGateway)
		return
	}

	resp := modelListResponse{Object: "list", Data: make([]modelInfo, 0, len(freeIDs))}
	for _, id := range freeIDs {
		resp.Data = append(resp.Data, modelInfo{ID: id, Object: "model", OwnedBy: "opencode-zen-free"})
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_ = json.NewEncoder(w).Encode(resp)
	p.logger.Printf("path=%s model=- upstream=intercepted free_count=%d", r.URL.Path, len(freeIDs))
}

// handlePost enforces the free-only rule on any POST that carries a model field
// (chat/completions, responses, embeddings, …), rewrites the model id to its bare
// form, then forwards the request upstream. Requests without a model field are
// forwarded untouched.
func (p *proxy) handlePost(w http.ResponseWriter, r *http.Request, normalized string) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "failed to read request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	var reqBody map[string]interface{}
	_ = json.Unmarshal(body, &reqBody)

	rawModel, _ := reqBody["model"].(string)
	model := stripProviderPrefix(rawModel)
	if model != "" {
		freeIDs, err := p.freeModels()
		if err != nil {
			p.logger.Printf("path=%s model=%s error computing free models: %v", r.URL.Path, model, err)
			http.Error(w, "failed to compute free models: "+err.Error(), http.StatusBadGateway)
			return
		}
		if !contains(freeIDs, model) {
			msg := "model " + rawModel + " is not a free model; this proxy only serves free OpenCode Zen models"
			p.logger.Printf("path=%s model=%s upstream=- rejected=free-only", r.URL.Path, model)
			http.Error(w, msg, http.StatusBadRequest)
			return
		}
		reqBody["model"] = model
		if rewritten, err := json.Marshal(reqBody); err == nil {
			body = rewritten
		}
	}

	p.forward(w, r, normalized, body, model)
}

// stripProviderPrefix removes a leading "provider/" segment some clients send
// (e.g. "zenfree/hy3-free"); OpenCode Zen expects the bare model id ("hy3-free").
func stripProviderPrefix(id string) string {
	if i := strings.IndexByte(id, '/'); i >= 0 {
		return id[i+1:]
	}
	return id
}

// hopHeaders are not forwarded to the upstream; they are connection-specific or
// would conflict with the rebuilt request. Keys are lower-cased: lookups must use
// strings.ToLower(k).
var hopHeaders = map[string]bool{
	"authorization":     true,
	"content-length":    true,
	"connection":        true,
	"transfer-encoding": true,
	"trailer":           true,
	"host":              true,
	"accept-encoding":   true,
}

// forward copies the incoming request (minus credential and hop-by-hop headers) to
// the upstream OpenCode Zen gateway, overlaying the OpenCode client identity so the
// request mirrors one sent by the official `opencode` CLI. Any header the client
// sends (including protocol-version markers) is preserved, so the proxy stays
// compatible with the current Zen protocol; only the key is stripped and the
// opencode identity is asserted.
func (p *proxy) forward(w http.ResponseWriter, r *http.Request, normalized string, body []byte, model string) {
	target := p.config.upstream + normalized

	var bodyReader io.Reader
	switch {
	case body != nil:
		bodyReader = bytes.NewReader(body)
	case r.Body != nil:
		bodyReader = r.Body
	}

	upstreamReq, err := http.NewRequestWithContext(r.Context(), r.Method, target, bodyReader)
	if err != nil {
		http.Error(w, "failed to build upstream request: "+err.Error(), http.StatusBadGateway)
		return
	}

	for k, vals := range r.Header {
		if hopHeaders[strings.ToLower(k)] {
			continue
		}
		for _, v := range vals {
			upstreamReq.Header.Add(k, v)
		}
	}

	upstreamReq.Header.Set("User-Agent", userAgent)
	upstreamReq.Header.Set("x-opencode-client", clientHeader)
	upstreamReq.Header.Set("x-opencode-session", randomHex(sessionHexLen))
	upstreamReq.Header.Set("x-opencode-project", randomHex(sessionHexLen))
	upstreamReq.Header.Set("x-opencode-request", randomHex(sessionHexLen))

	resp, err := p.client.Do(upstreamReq)
	if err != nil {
		p.logger.Printf("path=%s model=%s upstream=error: %v", r.URL.Path, model, err)
		http.Error(w, "upstream request failed: "+err.Error(), http.StatusBadGateway)
		return
	}
	defer resp.Body.Close()

	for k, vals := range resp.Header {
		if hopHeaders[strings.ToLower(k)] {
			continue
		}
		for _, v := range vals {
			w.Header().Add(k, v)
		}
	}
	w.WriteHeader(resp.StatusCode)
	if flusher, ok := w.(http.Flusher); ok {
		flusher.Flush()
	}
	_, _ = io.Copy(flushWriter{w}, resp.Body)

	p.logger.Printf("path=%s model=%s upstream=%d", r.URL.Path, model, resp.StatusCode)
}

type flushWriter struct {
	w http.ResponseWriter
}

func (fw flushWriter) Write(p []byte) (int, error) {
	n, err := fw.w.Write(p)
	if flusher, ok := fw.w.(http.Flusher); ok {
		flusher.Flush()
	}
	return n, err
}

func (p *proxy) freeModels() ([]string, error) {
	liveIDs, err := p.fetchLiveModels()
	if err != nil {
		return nil, err
	}
	costInfo := p.costInfo()

	free := make([]string, 0, len(liveIDs))
	for _, id := range liveIDs {
		if isFree(id, costInfo) {
			free = append(free, id)
		}
	}
	return free, nil
}

// costInfo returns models.dev cost info, using a short-lived cache. If models.dev
// is unreachable, it returns an empty map (non-fatal) so free detection falls back
// to the "-free" suffix rule, and the live model list still gates availability.
func (p *proxy) costInfo() map[string]costEntry {
	if info, ok := p.cost.get(); ok {
		return info
	}
	info, err := p.fetchCostInfo()
	if err != nil {
		p.logger.Printf("models.dev unavailable (%v); falling back to -free suffix rule", err)
		return map[string]costEntry{}
	}
	p.cost.set(info)
	return info
}

type costEntry struct {
	inputCost float64
	status    string
}

func (p *proxy) fetchLiveModels() ([]string, error) {
	req, err := http.NewRequest(http.MethodGet, p.config.upstream+"/models", nil)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Accept", "application/json")
	resp, err := p.client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	var list upstreamModelList
	if err := json.NewDecoder(resp.Body).Decode(&list); err != nil {
		return nil, err
	}
	ids := make([]string, 0, len(list.Data))
	for _, m := range list.Data {
		if m.ID != "" {
			ids = append(ids, m.ID)
		}
	}
	return ids, nil
}

func (p *proxy) fetchCostInfo() (map[string]costEntry, error) {
	req, err := http.NewRequest(http.MethodGet, p.config.modelsDev, nil)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Accept", "application/json")
	resp, err := p.client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	var parsed modelsDevResponse
	if err := json.NewDecoder(resp.Body).Decode(&parsed); err != nil {
		return nil, err
	}
	info := make(map[string]costEntry, len(parsed.OpenCode.Models))
	for id, entry := range parsed.OpenCode.Models {
		info[id] = costEntry{
			inputCost: entry.Cost.Input,
			status:    entry.Status,
		}
	}
	return info, nil
}

func isFree(id string, info map[string]costEntry) bool {
	if entry, ok := info[id]; ok {
		if entry.inputCost == 0 && entry.status != "deprecated" {
			return true
		}
	}
	return strings.HasSuffix(id, "-free")
}

func contains(items []string, target string) bool {
	for _, item := range items {
		if item == target {
			return true
		}
	}
	return false
}

func randomHex(length int) string {
	buf := make([]byte, (length+1)/2)
	if _, err := rand.Read(buf); err != nil {
		return strings.Repeat("0", length)
	}
	return hex.EncodeToString(buf)[:length]
}
