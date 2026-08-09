import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";

export interface Project {
  path: string;
  name: string;
  icon: string;
}

export interface ThemeDef {
  id: string;
  label: string;
  colors: Record<string, string>;
}

const THEME_COLOR_KEYS = [
  "slate-950", "slate-900", "slate-800", "slate-700", "slate-600",
  "slate-500", "slate-400", "slate-300", "slate-200", "slate-100",
  "emerald-300", "emerald-400", "emerald-500", "emerald-600", "emerald-700",
  "amber-100", "amber-200", "amber-300", "amber-400", "amber-500",
  "amber-800", "amber-900", "amber-950",
  "red-300", "red-800", "red-950",
  "cyan-300", "cyan-400",
];

export function humanizeProjectName(path: string): string {
  const base = path.split("/").filter(Boolean).pop() ?? path;
  const words = base.replace(/[-_]+/g, " ").trim().split(" ");
  const first = words[0] ?? base;
  const rest = words.length > 1 ? " " + words.slice(1).join(" ") : "";
  return first.charAt(0).toUpperCase() + first.slice(1) + rest;
}

export const useProjectsStore = defineStore("projects", {
  state: () => ({
    projects: [] as Project[],
    theme: "one-dark-pro",
    themes: [] as ThemeDef[],
    loaded: false,
  }),
  actions: {
    async load() {
      try {
        const cfg = await invoke<{ theme: string; projects: Project[] }>("load_config");
        this.theme = cfg?.theme || "one-dark-pro";
        this.projects = Array.isArray(cfg?.projects) ? cfg.projects : [];
      } catch (err) {
        console.error("load_config", err);
        this.projects = [];
      }
      await this.loadThemes();
      this.loaded = true;
    },
    async loadThemes() {
      try {
        this.themes = await invoke<ThemeDef[]>("list_themes");
      } catch (err) {
        console.error("list_themes", err);
      }
      this.applyTheme();
    },
    applyTheme() {
      document.documentElement.dataset.theme = this.theme;
      const found = this.themes.find((t) => t.id === this.theme);
      const root = document.documentElement.style;
      for (const key of THEME_COLOR_KEYS) {
        const value = found?.colors[key];
        if (value) root.setProperty(`--color-${key}`, value);
        else root.removeProperty(`--color-${key}`);
      }
    },
    async setTheme(theme: string) {
      if (!theme || this.theme === theme) return;
      this.theme = theme;
      this.applyTheme();
      await this.persist();
    },
    async persist() {
      try {
        await invoke("save_config", { theme: this.theme, projects: this.projects });
      } catch (err) {
        console.error("save_config", err);
      }
    },
    async upsert(path: string) {
      if (!path) return;
      const existing = this.projects.find((p) => p.path === path);
      if (existing) {
        this.projects = [existing, ...this.projects.filter((p) => p.path !== path)];
      } else {
        this.projects = [
          { path, name: humanizeProjectName(path), icon: "📂" },
          ...this.projects,
        ];
      }
      await this.persist();
    },
    async update(path: string, patch: { name?: string; icon?: string }) {
      const idx = this.projects.findIndex((p) => p.path === path);
      if (idx === -1) return;
      if (patch.icon !== undefined) {
        this.projects[idx].icon = patch.icon.trim() || "📂";
      }
      if (patch.name !== undefined) {
        this.projects[idx].name = patch.name.trim() || humanizeProjectName(path);
      }
      await this.persist();
    },
    async remove(path: string) {
      this.projects = this.projects.filter((p) => p.path !== path);
      await this.persist();
    },
  },
});
