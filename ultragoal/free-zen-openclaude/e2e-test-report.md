# E2E Test Report: free-zen.py + openclaude

Date: 2026-07-20 07:08:21 UTC

## Environment

- Python: 3.12.3
- OpenClaude: 0.24.0 (OpenClaude)
- OS: Linux

## Results Summary

- **Passed: 26**
- **Failed: 0**
- **Total:  26**

## Test Details

### 1. Unit Tests

**Unit tests (15)**: `PASS`
```
All 15 passed
```
### 2. Proxy Model Filtering

**Proxy returns only free models**: `PASS`
```
Proxy: 6 models, Real Zen: 55 models
Filtered IDs: {'deepseek-v4-flash-free', 'nemotron-3-ultra-free', 'big-pickle', 'mimo-v2.5-free', 'north-mini-code-free', 'hy3-free'}
```
**Proxy count ≤ 6**: `PASS`
```
Got 6 models
```
**All free models included (6/6)**: `PASS`
```
Expected 6 free models, got 6: {'deepseek-v4-flash-free', 'nemotron-3-ultra-free', 'big-pickle', 'mimo-v2.5-free', 'north-mini-code-free', 'hy3-free'}
```
**Paid models excluded (0 leaked)**: `PASS`
```
49 paid models filtered out (55 - 6)
```
### 3. Header Injection

**Headers contain all 5 required keys**: `PASS`
```
Got keys: {'x-opencode-client', 'User-Agent', 'x-opencode-request', 'x-opencode-session', 'x-opencode-project'}
```
**User-Agent format correct**: `PASS`
```
User-Agent: opencode/latest/1.3.15/cli
```
**All 3 IDs are 26-char hex**: `PASS`
```
session=719322014c124f5495c6c4e90b, project=2778a36bee8a49b1bb39856d2c, request=3a23718554364236a4d4e930af
```
**IDs are unique per call**: `PASS`
```
Session IDs differ
```
### 4. Chat Completion

**Chat completion returns choices**: `PASS`
```
Choices count: 1
```
**Chat completion cost is 0**: `PASS`
```
Cost: 0
```
**Response matches expectation**: `PASS`
```
Response: hello world
```
### 5. OpenClaude Integration

**OpenClaude runs without errors**: `PASS`
```
Exit: 0
```
**OpenClaude returns correct response**: `PASS`
```
Stdout: hello
```
**OpenClaude uses free model**: `PASS`
```
Response: deepseek-v4-flash-free
```
### 6. Error Handling

**Invalid port (privileged) -> PermissionError**: `PASS`
```
Tested at OS level (port 1)
```
**Invalid model returns upstream error**: `PASS`
```
Status: 401, Body: {"type":"error","error":{"type":"ModelError","message":"Model nonexistent-model-xyz is not supported"}}
```
**Port already in use raises OSError**: `PASS`
```
OSError: [Errno 98] Address already in use
```
### 7. Edge Cases

**--list flag works**: `PASS`
```
RC: 0
```
**--list shows names (no JSON)**: `PASS`
```
Output: 'DeepSeek V4 Flash Free\nMiMo V2.5 Free'
```
**--json returns valid JSON with IDs**: `PASS`
```
2 models: ['deepseek-v4-flash-free', 'mimo-v2.5-free']
```
**Env export format correct**: `PASS`
```
Contains all 5 env vars: True
```
**Best model selected (deepseek-v4-flash-free)**: `PASS`
```
Model line: ['OPENAI_MODEL=deepseek-v4-flash-free']
```
**Fallback to hardcoded list when fetch fails**: `PASS`
```
Uses fallback model
```
**--no-color flag works**: `PASS`
```
Listed with no color
```
**Launch with missing binary returns code 2**: `PASS`
```
Exit code: 2
```