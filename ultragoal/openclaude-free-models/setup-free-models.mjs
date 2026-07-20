#!/usr/bin/env node

import { createHash, randomUUID } from 'node:crypto';
import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'node:fs';
import { homedir, platform } from 'node:os';
import { join } from 'node:path';

const ZEN_BASE_URL = 'https://opencode.ai/zen/v1';
const MODELS_DEV_URL = 'https://models.dev/api.json';

const OPENCLAUDE_CONFIG_DIR = join(homedir(), '.openclaude');
const PROFILE_PATH = join(OPENCLAUDE_CONFIG_DIR, '.openclaude-profile.json');

function opencodeHeaders() {
  const id = () => randomUUID().replace(/-/g, '').slice(0, 26);
  return {
    'User-Agent': 'opencode/latest/1.3.15/cli',
    'x-opencode-client': 'cli',
    'x-opencode-session': id(),
    'x-opencode-project': id(),
    'x-opencode-request': id(),
  };
}

async function fetchFreeModels() {
  const [modelsDevResp] = await Promise.all([
    fetch(MODELS_DEV_URL),
  ]);

  if (!modelsDevResp.ok) {
    throw new Error(`models.dev API returned ${modelsDevResp.status}`);
  }

  const modelsDev = await modelsDevResp.json();
  const opencodeModels = modelsDev.opencode?.models;
  if (!opencodeModels) {
    throw new Error('No opencode models section found in models.dev response');
  }

      const freeModels = [];
  for (const [id, info] of Object.entries(opencodeModels)) {
    if (info.status === 'deprecated') continue;
    const cost = info.cost;
    if (!cost || cost.input !== 0) continue;
    freeModels.push({
      id,
      name: info.name || id,
      description: info.description || '',
      contextWindow: info.limit?.context || 128000,
      maxTokens: info.limit?.output || 64000,
      reasoning: info.reasoning || false,
      input: info.modalities?.input || ['text'],
    });
  }

  freeModels.sort((a, b) => a.name.localeCompare(b.name));

  // prefer well-known models as the first default
  const preferredDefault = freeModels.find(m =>
    m.id.includes('deepseek') || m.id.includes('nemotron')
  );

  return { models: freeModels, preferredDefault: preferredDefault?.id || freeModels[0]?.id };
}

function loadOrCreateProfile() {
  if (existsSync(PROFILE_PATH)) {
    try {
      return JSON.parse(readFileSync(PROFILE_PATH, 'utf8'));
    } catch {
      return null;
    }
  }
  return null;
}

function saveProfile(profile) {
  if (!existsSync(OPENCLAUDE_CONFIG_DIR)) {
    mkdirSync(OPENCLAUDE_CONFIG_DIR, { recursive: true, mode: 0o700 });
  }
  writeFileSync(PROFILE_PATH, JSON.stringify(profile, null, 2), { mode: 0o600 });
  console.log(`\nProfile saved to: ${PROFILE_PATH}`);
}

function main() {
  const args = process.argv.slice(2);
  const apiKey = args.find(a => a.startsWith('--api-key='))?.split('=')[1] || process.env.OPENCODE_API_KEY || 'public';

  fetchFreeModels()
    .then(({ models, preferredDefault }) => {
      if (models.length === 0) {
        console.log('No free models found.');
        process.exit(1);
      }

      console.log(`\nFound ${models.length} free OpenCode Zen models:\n`);
      const table = models.map((m, i) => {
        const reasoning = m.reasoning ? '🧠' : '  ';
        return `${String(i + 1).padStart(2)}. ${reasoning} ${m.id.padEnd(35)} ${m.name.padEnd(30)} ctx:${String(m.contextWindow).padStart(7)} out:${String(m.maxTokens).padStart(6)}`;
      }).join('\n');
      console.log(table);

      const defaultModel = preferredDefault;
      const existingProfile = loadOrCreateProfile();

      const profile = {
        profile: 'opencode',
        env: {
          OPENCODE_API_KEY: apiKey,
          OPENAI_BASE_URL: ZEN_BASE_URL,
          OPENAI_MODEL: existingProfile?.env?.OPENAI_MODEL || defaultModel,
        },
        createdAt: existingProfile?.createdAt || new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      saveProfile(profile);

      console.log(`\nDefault model: ${profile.env.OPENAI_MODEL}`);
      console.log(`API key: ${apiKey === 'public' ? 'public (free only)' : 'configured'}`);
      console.log('\nRun `openclaude` to start with OpenCode Zen free models.');
      console.log('Use `/model` inside openclaude to switch between free models.\n');
    })
    .catch(err => {
      console.error('Error:', err.message);
      process.exit(1);
    });
}

main();
