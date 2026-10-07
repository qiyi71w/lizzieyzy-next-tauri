import { describe, expect, it } from "vitest";
import { baseResources, createTranslator, resolveEffectiveLocale } from "./resources";

describe("first-consumer resources", () => {
  it("retains effective Chinese and falls back to the complete base, never a key", () => {
    expect(resolveEffectiveLocale()).toBe("zh-CN");
    expect(resolveEffectiveLocale("en_XX")).toBe("zh-CN");
    const t = createTranslator("en", { en: { "search.title": "Find a function" } });
    expect(t("search.title")).toBe("Find a function");
    expect(t("search.empty")).toBe("没有匹配的功能或设置。");
    for (const key of Object.keys(baseResources) as Array<keyof typeof baseResources>) {
      expect(t(key)).not.toBe(key);
      expect(t(key).trim()).not.toBe("");
    }
  });
});
