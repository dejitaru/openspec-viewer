export interface Scenario {
  name: string;
  body: string;
}

export interface Requirement {
  name: string;
  description: string;
  scenarios: Scenario[];
}

export interface ParsedSpec {
  purpose: string;
  requirements: Requirement[];
  topScenarios: Scenario[];
  limitations: string;
  nonGoals: string;
}

function normalizeSection(title: string): string {
  const t = title.toLowerCase().replace(/[`*]/g, "").trim();
  if (/non-goals?|no[- ]objetivos?|out of scope|fuera de alcance/.test(t)) return "nongoals";
  if (/known limitations?|limitaciones? conocidas?|limitaciones?/.test(t)) return "limitations";
  if (/purpose|propósito|proposito|descripci[oó]n|objetivo general/.test(t)) return "purpose";
  if (/requirements?|requisitos?|requerimientos?/.test(t)) return "requirements";
  if (/scenarios?|escenarios?/.test(t)) return "scenarios";
  return "";
}

export function parseSpec(content: string): ParsedSpec {
  const requirements: Requirement[] = [];
  const topScenarios: Scenario[] = [];
  const rawSections: Record<string, string[]> = {};

  let section = "";
  let req: Requirement | null = null;
  let scen: Scenario | null = null;
  let pending: string[] = [];

  const joinText = (a: string, b: string) => (a ? a + "\n" + b : b);

  function flushPending() {
    const text = pending.join("\n").replace(/\n{3,}/g, "\n\n").trim();
    if (text) {
      if (scen) scen.body = joinText(scen.body, text);
      else if (req) req.description = joinText(req.description, text);
    }
    pending = [];
  }

  for (const line of content.split("\n")) {
    const h2 = line.match(/^##\s+(.+)/);
    const reqHead = line.match(/^#{3,}\s*(?:Requirement|Requisito|Requerimiento):\s*(.*)/i);
    const scenHead = line.match(/^#{3,}\s*(?:Scenario|Escenario):\s*(.*)/i);

    if (reqHead) {
      flushPending();
      req = { name: reqHead[1].trim(), description: "", scenarios: [] };
      scen = null;
      requirements.push(req);
      continue;
    }
    if (scenHead) {
      flushPending();
      scen = { name: scenHead[1].trim(), body: "" };
      if (req) req.scenarios.push(scen);
      else topScenarios.push(scen);
      continue;
    }
    if (h2) {
      flushPending();
      section = normalizeSection(h2[1]);
      req = null;
      scen = null;
      continue;
    }
    if (section && !line.trim().startsWith("#")) {
      rawSections[section] = rawSections[section] || [];
      rawSections[section].push(line);
    }
    pending.push(line);
  }
  flushPending();

  if (requirements.length === 0 && rawSections["requirements"]) {
    for (const line of rawSections["requirements"]) {
      const item = line.match(/^\s*-\s*(?:\[[ xX]\]\s*)?(.+)/);
      if (item && item[1].trim()) {
        requirements.push({ name: item[1].trim(), description: "", scenarios: [] });
      }
    }
  }

  return {
    purpose: (rawSections["purpose"] || []).join("\n").trim(),
    requirements,
    topScenarios,
    limitations: (rawSections["limitations"] || []).join("\n").trim(),
    nonGoals: (rawSections["nongoals"] || []).join("\n").trim(),
  };
}
