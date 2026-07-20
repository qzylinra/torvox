# Stage 3: Test Plan

## Tests

### T1: Script execution
- Run `node setup-free-models.mjs`
- Expect: exit 0, prints model list, saves profile
- Result: ✓

### T2: Profile structure
- Check `~/.openclaude/.openclaude-profile.json`
- Expect: valid JSON with `profile: "opencode"`, env vars for base URL, API key, model
- Result:

### T3: openclaude startup
- Run `openclaude -p "Output only the word OK"`
- Expect: returns "OK" using OpenCode Zen free models
- Result:

### T4: Model switch
- Run with `--model deepseek-v4-flash-free` 
- Expect: uses the specified model
- Result:

### T5: Direct API verification
- Call `POST /v1/chat/completions` with Bearer public
- Expect: 200 response, cost=0
- Result:

### T6: Re-run preserves model
- Run the script twice, verify existing model is preserved
- Result:
