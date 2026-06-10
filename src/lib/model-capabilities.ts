import { hasCapability } from "./ollama";
import type { ModelInfo } from "./types";

export function modelsWithCapability(
  models: ModelInfo[],
  capability: string,
): ModelInfo[] {
  return models.filter((m) => hasCapability(m, capability));
}

export function suggestModels(
  models: ModelInfo[],
  capability: string,
  limit = 3,
): string[] {
  return modelsWithCapability(models, capability)
    .slice(0, limit)
    .map((m) => m.name);
}
