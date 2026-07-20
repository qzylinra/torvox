package main

import (
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
	case r.Method == http.MethodPost && normalized == "/chat/completions":
		p.handleChatCompletions(w, r, normalized)
	default:
		http.Error(w, "only GET /v1/models and POST /v1/chat/completions are supported", http.StatusNotFound)
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

func (p *proxy) handleChatCompletions(w http.ResponseWriter, r *http.Request, normalized string) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "failed to read request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	var parsed struct {
		Model string `json:"model"`
	}
	if err := json.Unmarshal(body, &parsed); err != nil {
		http.Error(w, "malformed JSON in request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	freeIDs, err := p.freeModels()
	if err != nil {
		p.logger.Printf("path=%s model=%s error computing free models: %v", r.URL.Path, parsed.Model, err)
		http.Error(w, "failed to compute free models: "+err.Error(), http.StatusBadGateway)
		return
	}

	if !contains(freeIDs, parsed.Model) {
		msg := "model " + parsed.Model + " is not a free model; this proxy only serves free OpenCode Zen models"
		p.logger.Printf("path=%s model=%s upstream=- rejected=free-only", r.URL.Path, parsed.Model)
		http.Error(w, msg, http.StatusBadRequest)
		return
	}

	p.forward(w, r, normalized, body, parsed.Model)
}

func (p *proxy) forward(w http.ResponseWriter, r *http.Request, normalized string, body []byte, model string) {
	target := p.config.upstream + normalized

	upstreamReq, err := http.NewRequestWithContext(r.Context(), r.Method, target, strings.NewReader(string(body)))
	if err != nil {
		http.Error(w, "failed to build upstream request: "+err.Error(), http.StatusBadGateway)
		return
	}

	if contentType := r.Header.Get("Content-Type"); contentType != "" {
		upstreamReq.Header.Set("Content-Type", contentType)
	}
	if accept := r.Header.Get("Accept"); accept != "" {
		upstreamReq.Header.Set("Accept", accept)
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

	if contentType := resp.Header.Get("Content-Type"); contentType != "" {
		w.Header().Set("Content-Type", contentType)
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
