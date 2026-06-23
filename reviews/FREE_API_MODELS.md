# Free & API Models — OpenRouter & NVIDIA NIM

Generated: 2026-06-23 20:33 UTC

## OpenRouter — zero-cost models (`prompt` and `completion` = $0)

| Model ID | Name | Context |
|----------|------|---------|
| `cognitivecomputations/dolphin-mistral-24b-venice-edition:free` | Venice: Uncensored (free) | 32768 |
| `cohere/north-mini-code:free` | Cohere: North Mini Code (free) | 256000 |
| `google/gemma-4-26b-a4b-it:free` | Google: Gemma 4 26B A4B  (free) | 262144 |
| `google/gemma-4-31b-it:free` | Google: Gemma 4 31B (free) | 262144 |
| `google/lyria-3-clip-preview` | Google: Lyria 3 Clip Preview | 1048576 |
| `google/lyria-3-pro-preview` | Google: Lyria 3 Pro Preview | 1048576 |
| `liquid/lfm-2.5-1.2b-instruct:free` | LiquidAI: LFM2.5-1.2B-Instruct (free) | 32768 |
| `liquid/lfm-2.5-1.2b-thinking:free` | LiquidAI: LFM2.5-1.2B-Thinking (free) | 32768 |
| `meta-llama/llama-3.2-3b-instruct:free` | Meta: Llama 3.2 3B Instruct (free) | 131072 |
| `meta-llama/llama-3.3-70b-instruct:free` | Meta: Llama 3.3 70B Instruct (free) | 131072 |
| `nousresearch/hermes-3-llama-3.1-405b:free` | Nous: Hermes 3 405B Instruct (free) | 131072 |
| `nvidia/nemotron-3-nano-30b-a3b:free` | NVIDIA: Nemotron 3 Nano 30B A3B (free) | 256000 |
| `nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free` | NVIDIA: Nemotron 3 Nano Omni (free) | 256000 |
| `nvidia/nemotron-3-super-120b-a12b:free` | NVIDIA: Nemotron 3 Super (free) | 1000000 |
| `nvidia/nemotron-3-ultra-550b-a55b:free` | NVIDIA: Nemotron 3 Ultra (free) | 1000000 |
| `nvidia/nemotron-3.5-content-safety:free` | NVIDIA: Nemotron 3.5 Content Safety (free) | 128000 |
| `nvidia/nemotron-nano-12b-v2-vl:free` | NVIDIA: Nemotron Nano 12B 2 VL (free) | 128000 |
| `nvidia/nemotron-nano-9b-v2:free` | NVIDIA: Nemotron Nano 9B V2 (free) | 128000 |
| `openai/gpt-oss-120b:free` | OpenAI: gpt-oss-120b (free) | 131072 |
| `openai/gpt-oss-20b:free` | OpenAI: gpt-oss-20b (free) | 131072 |
| `openrouter/free` | Free Models Router | 200000 |
| `openrouter/owl-alpha` | Owl Alpha | 1048756 |
| `poolside/laguna-m.1:free` | Poolside: Laguna M.1 (free) | 262144 |
| `poolside/laguna-xs.2:free` | Poolside: Laguna XS.2 (free) | 262144 |
| `qwen/qwen3-coder:free` | Qwen: Qwen3 Coder 480B A35B (free) | 1048576 |
| `qwen/qwen3-next-80b-a3b-instruct:free` | Qwen: Qwen3 Next 80B A3B Instruct (free) | 262144 |

**Count:** 26 models with explicit $0 pricing on OpenRouter.

### Review models used for gnomad-slate Linux audit

| Role | Model ID | Notes |
|------|----------|-------|
| Primary reasoning | `nvidia/nemotron-3-ultra-550b-a55b:free` | Nemotron 3 Ultra (free tier on OpenRouter) |
| Architecture / GLM 5 | `z-ai/glm-5.2` | Latest GLM-5 family on OpenRouter |
| Code specialist | `qwen/qwen3-coder:free` | Large coding model, free on OpenRouter |

---

## NVIDIA NIM — available models via API

NIM does not expose per-token $0 flags in the `/v1/models` listing. Access is via `NVIDIA_API_KEY` at `https://integrate.api.nvidia.com/v1`. Many models are included in NVIDIA's developer free-tier credits; confirm current quotas in the [NVIDIA API catalog](https://build.nvidia.com/).

**Total models in catalog:** 121

### Nemotron family (NIM)

| Model ID |
|----------|
| `mistralai/mistral-nemotron` |
| `nvidia/llama-3.1-nemotron-51b-instruct` |
| `nvidia/llama-3.1-nemotron-70b-instruct` |
| `nvidia/llama-3.1-nemotron-nano-8b-v1` |
| `nvidia/llama-3.1-nemotron-nano-vl-8b-v1` |
| `nvidia/llama-3.1-nemotron-safety-guard-8b-v3` |
| `nvidia/llama-3.1-nemotron-ultra-253b-v1` |
| `nvidia/llama-3.3-nemotron-super-49b-v1` |
| `nvidia/llama-3.3-nemotron-super-49b-v1.5` |
| `nvidia/llama-nemotron-embed-1b-v2` |
| `nvidia/llama-nemotron-embed-vl-1b-v2` |
| `nvidia/nemotron-3-content-safety` |
| `nvidia/nemotron-3-nano-30b-a3b` |
| `nvidia/nemotron-3-nano-omni-30b-a3b-reasoning` |
| `nvidia/nemotron-3-super-120b-a12b` |
| `nvidia/nemotron-3-ultra-550b-a55b` |
| `nvidia/nemotron-3.5-content-safety` |
| `nvidia/nemotron-4-340b-instruct` |
| `nvidia/nemotron-4-340b-reward` |
| `nvidia/nemotron-content-safety-reasoning-4b` |
| `nvidia/nemotron-mini-4b-instruct` |
| `nvidia/nemotron-nano-12b-v2-vl` |
| `nvidia/nemotron-nano-3-30b-a3b` |
| `nvidia/nemotron-parse` |
| `nvidia/nvidia-nemotron-nano-9b-v2` |

### GLM family (NIM)

| Model ID |
|----------|
| `z-ai/glm-5.1` |

### Notable NIM equivalents for this review

| Purpose | OpenRouter | NVIDIA NIM |
|---------|------------|------------|
| Nemotron 3 Ultra | `nvidia/nemotron-3-ultra-550b-a55b:free` | `nvidia/nemotron-3-ultra-550b-a55b` |
| GLM 5 latest | `z-ai/glm-5.2` | `z-ai/glm-5.1` (latest on NIM catalog) |
| Code review alt | `qwen/qwen3-coder:free` | `deepseek-ai/deepseek-v4-flash` |

---

## How to refresh this list

```bash
# OpenRouter free models
curl -s https://openrouter.ai/api/v1/models -H "Authorization: Bearer $OPENROUTER_API_KEY" | jq '[.data[] | select((.pricing.prompt|tonumber)==0 and (.pricing.completion|tonumber)==0) | .id]'

# NVIDIA NIM catalog
curl -s https://integrate.api.nvidia.com/v1/models -H "Authorization: Bearer $NVIDIA_API_KEY" | jq '.data[].id'
```
