# Stage 3: Test Results

## T1: Script execution
```
$ node ultragoal/openclaude-free-models/setup-free-models.mjs

Found 6 free OpenCode Zen models:

 1. 🧠 big-pickle                          Big Pickle                     ctx: 200000 out: 32000
 2. 🧠 deepseek-v4-flash-free              DeepSeek V4 Flash Free         ctx: 200000 out:128000
 3. 🧠 hy3-free                            Hy3 Free                       ctx: 190000 out: 64000
 4. 🧠 mimo-v2.5-free                      MiMo V2.5 Free                 ctx: 200000 out: 32000
 5. 🧠 nemotron-3-ultra-free               Nemotron 3 Ultra Free          ctx:1000000 out:128000
 6. 🧠 north-mini-code-free                North Mini Code Free           ctx: 256000 out: 64000

Profile saved to: /home/runner/.openclaude/.openclaude-profile.json
Default model: deepseek-v4-flash-free
API key: public (free only)
```
**PASS** ✓

## T2: Profile structure
```json
{
  "profile": "opencode",
  "env": {
    "OPENCODE_API_KEY": "public",
    "OPENAI_BASE_URL": "https://opencode.ai/zen/v1",
    "OPENAI_MODEL": "deepseek-v4-flash-free"
  },
  "createdAt": "2026-07-20T05:52:11.347Z",
  "updatedAt": "2026-07-20T05:52:11.347Z"
}
```
**PASS** ✓ — Valid JSON, correct structure, uses standard openclaude profile format.

## T3: openclaude startup
```
$ timeout 60 openclaude -p "Output only the word OK"
OK
```
**PASS** ✓ — openclaude starts, loads profile, makes API calls to OpenCode Zen, and responds.

## T4: Model switch
```
$ timeout 60 openclaude -p "Output only the word OK" --model big-pickle
OK
```
**PASS** ✓ — Model override works.

## T5: Direct API verification
```
POST https://opencode.ai/zen/v1/chat/completions
Authorization: Bearer public
Body: { model: "big-pickle", messages: [{ role: "user", content: "Say hello in one word" }] }

Response: 200 OK, cost: 0
```
**PASS** ✓ — OpenCode Zen free tier works with just `Authorization: Bearer public`.

## T6: Re-run preserves model
```
$ node ultragoal/openclaude-free-models/setup-free-models.mjs
...
Default model: deepseek-v4-flash-free  (preserved from previous session)
```
**PASS** ✓ — Script preserves the user's existing model choice.

## Summary
All 6 tests pass. The setup script correctly:
1. Discovers 6 free models from OpenCode Zen
2. Creates a valid openclaude profile
3. Works with openclaude's existing provider system
4. The OpenCode Zen public API (Bearer public) works without any API key

## Free Models Available
| # | Model ID | Name | Context | Max Output | Reasoning |
|---|----------|------|---------|------------|-----------|
| 1 | big-pickle | Big Pickle | 200K | 32K | ✓ |
| 2 | deepseek-v4-flash-free | DeepSeek V4 Flash Free | 200K | 128K | ✓ |
| 3 | hy3-free | Hy3 Free | 190K | 64K | ✓ |
| 4 | mimo-v2.5-free | MiMo V2.5 Free | 200K | 32K | ✓ |
| 5 | nemotron-3-ultra-free | Nemotron 3 Ultra Free | 1M | 128K | ✓ |
| 6 | north-mini-code-free | North Mini Code Free | 256K | 64K | ✓ |
