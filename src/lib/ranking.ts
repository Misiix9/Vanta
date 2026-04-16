/**
 * Scoring and ranking logic for search results, commands, and macros.
 * Extracted from +page.svelte to keep all ranking weights in one place.
 */

import type { SearchResult, VantaConfig, WorkflowMacro } from "$lib/types";

export const builtinCommands: SearchResult[] = [
  {
    title: "Feature Hub",
    subtitle: "Open the guided hub for sharing, templates, and workflows",
    icon: "fa-solid fa-compass",
    exec: "open-window:featureHub",
    score: 1_450_000,
    match_indices: [],
    source: "Application",
    id: "discover-feature-hub",
    section: "Discover",
  },
  {
    title: "Community Sharing",
    subtitle: "Feedback, roadmap voting, snippets, and popular workflows",
    icon: "fa-solid fa-users",
    exec: "open-window:communityHub",
    score: 1_445_000,
    match_indices: [],
    source: "Application",
    id: "discover-community-sharing",
    section: "Discover",
  },
  {
    title: "Theme And Profile Studio",
    subtitle: "Jump directly to theme profile editing",
    icon: "fa-solid fa-palette",
    exec: "open-window:themeHub",
    score: 1_440_000,
    match_indices: [],
    source: "Application",
    id: "discover-theme-profile",
    section: "Discover",
  },
  {
    title: "Open Extensions Store",
    subtitle: "Browse and install extensions",
    icon: "fa-solid fa-store",
    exec: "open-store",
    score: 1_435_000,
    match_indices: [],
    source: "Application",
    id: "discover-store",
    section: "Discover",
  },
  {
    title: "Extension Template Wizard",
    subtitle: "Create a starter extension template in one click",
    icon: "fa-solid fa-wand-magic-sparkles",
    exec: "open-window:extensionsHub",
    score: 1_434_000,
    match_indices: [],
    source: "Application",
    id: "discover-extension-template",
    section: "Discover",
  },
  {
    title: "Sleep",
    subtitle: "Suspend this machine immediately",
    icon: "fa-solid fa-moon",
    exec: "system-action:sleep",
    score: 1_400_000,
    match_indices: [],
    source: "Application",
    id: "cmd-sleep",
    section: "Commands",
  },
  {
    title: "Lock",
    subtitle: "Lock the current session",
    icon: "fa-solid fa-lock",
    exec: "system-action:lock",
    score: 1_390_000,
    match_indices: [],
    source: "Application",
    id: "cmd-lock",
    section: "Commands",
  },
  {
    title: "Shutdown",
    subtitle: "Power off this device",
    icon: "fa-solid fa-power-off",
    exec: "system-action:shutdown",
    score: 1_380_000,
    match_indices: [],
    source: "Application",
    id: "cmd-shutdown",
    section: "Commands",
  },
  {
    title: "Restart",
    subtitle: "Reboot the machine",
    icon: "fa-solid fa-rotate-right",
    exec: "system-action:restart",
    score: 1_370_000,
    match_indices: [],
    source: "Application",
    id: "cmd-restart",
    section: "Commands",
  },
  {
    title: "Go to BIOS",
    subtitle: "Reboot into firmware setup",
    icon: "fa-solid fa-microchip",
    exec: "system-action:bios",
    score: 1_350_000,
    match_indices: [],
    source: "Application",
    id: "cmd-bios",
    section: "Commands",
  },
  {
    title: "Log Out",
    subtitle: "Terminate the current session",
    icon: "fa-solid fa-right-from-bracket",
    exec: "system-action:logout",
    score: 1_360_000,
    match_indices: [],
    source: "Application",
    id: "cmd-logout",
    section: "Commands",
  },
];

export function filterCommandResults(
  q: string,
  config: VantaConfig | undefined,
): SearchResult[] {
  if (config && config.search.applications.enabled === false) {
    return [];
  }

  const normalized = q.trim().toLowerCase();
  const appWeight = config?.search?.applications?.weight ?? 100;
  if (!normalized) {
    return builtinCommands.map((cmd) => ({
      ...cmd,
      score: Math.round((cmd.score * appWeight) / 100),
    }));
  }

  return builtinCommands
    .map((cmd) => {
      const title = cmd.title.toLowerCase();
      const subtitle = (cmd.subtitle || "").toLowerCase();

      let base = 0;
      if (title === normalized) base = 30_000;
      else if (title.startsWith(normalized)) base = 22_000;
      else if (title.includes(normalized)) base = 15_000;
      else if (subtitle.includes(normalized)) base = 9_000;

      if (!base) return null;

      return {
        ...cmd,
        score: Math.round((base * appWeight) / 100),
      } as SearchResult;
    })
    .filter((item): item is SearchResult => item !== null);
}

export function buildMacroResults(
  q: string,
  macros: WorkflowMacro[],
  config: VantaConfig | undefined,
): SearchResult[] {
  if (config && config.search.applications.enabled === false) return [];
  if (!macros.length) return [];
  const needle = q.trim().toLowerCase();
  const appWeight = config?.search?.applications?.weight ?? 100;
  return macros
    .filter((macro) => {
      if (!macro.enabled) return false;
      if (!needle) return true;
      return (
        macro.id.toLowerCase().includes(needle) ||
        macro.name.toLowerCase().includes(needle) ||
        (macro.description ?? "").toLowerCase().includes(needle)
      );
    })
    .map((macro, idx) => {
      let base = 1_300_000 - idx;
      if (needle) {
        const name = macro.name.toLowerCase();
        const desc = (macro.description ?? "").toLowerCase();
        if (name === needle) base = 28_000;
        else if (name.startsWith(needle)) base = 20_000;
        else if (name.includes(needle) || desc.includes(needle)) base = 13_000;
        else base = 9_000;
      }

      return {
        title: macro.name,
        subtitle: macro.description ?? macro.id,
        icon: "fa-solid fa-diagram-project",
        exec: `macro:${macro.id}`,
        score: Math.round((base * appWeight) / 100),
        match_indices: [],
        source: "Application",
        id: `macro-${macro.id}`,
        section: "Macros",
      } as SearchResult;
    });
}

export function composeResults(
  base: SearchResult[],
  q: string,
  macros: WorkflowMacro[],
  config: VantaConfig | undefined,
): SearchResult[] {
  const commands = filterCommandResults(q, config);
  const macroResults = buildMacroResults(q, macros, config);
  const merged = [...commands, ...macroResults, ...base];
  return merged.sort((a, b) => (b.score || 0) - (a.score || 0));
}

export function commandToExec(command: import("$lib/types").CommandContract): string {
  switch (command.kind) {
    case "launch_app":
      return command.exec;
    case "open_file":
      return command.path;
    case "open_settings":
      return "open-settings";
    case "open_settings_section":
      return `open-settings:${command.section}`;
    case "open_feature_window":
      return `open-window:${command.window}`;
    case "open_store":
      return "open-store";
    case "copy_text":
      return `copy:${command.value}`;
    case "copy_path":
      return `copy-path:${command.value}`;
    case "reveal_path":
      return `reveal:${command.path}`;
    case "open_with_editor":
      return `open-with:${command.path}`;
    case "focus_window":
      return `focus:${command.id}`;
    case "close_window":
      return `close-window:${command.id}`;
    case "minimize_window":
      return `minimize-window:${command.id}`;
    case "move_window_current_workspace":
      return `move-window-current:${command.id}`;
    case "system_action":
      return `system-action:${command.action}`;
    case "macro_open":
      return `macro:${command.id}`;
    case "macro_template_open":
      return `macro-template:${command.template_id}`;
    case "extension_view":
      return `ext-view:${command.ext_id}:${command.command}`;
    case "extension_action":
      return `ext-no-view:${command.ext_id}:${command.command}`;
    case "query_fill":
      return `fill:${command.value}`;
    case "intent_workflow":
      return `intent-workflow:${command.steps.join("|||")}`;
    case "profile_switch":
      return `profile-switch:${command.id}`;
    case "unknown":
      return command.exec;
  }
}

export function inferCommandFromResult(
  result: SearchResult,
): import("$lib/types").CommandContract {
  const exec = result.exec;
  if (exec.startsWith("system-action:")) {
    return { kind: "system_action", action: exec.slice(14) };
  }
  if (exec === "open-settings") return { kind: "open_settings" };
  if (exec.startsWith("open-settings:")) {
    return { kind: "open_settings_section", section: exec.slice(14) };
  }
  if (exec.startsWith("open-window:")) {
    return { kind: "open_feature_window", window: exec.slice(12) };
  }
  if (exec === "open-store") return { kind: "open_store" };
  if (exec.startsWith("copy:")) return { kind: "copy_text", value: exec.slice(5) };
  if (exec.startsWith("copy-path:")) return { kind: "copy_path", value: exec.slice(10) };
  if (exec.startsWith("reveal:")) return { kind: "reveal_path", path: exec.slice(7) };
  if (exec.startsWith("open-with:")) {
    return { kind: "open_with_editor", path: exec.slice(10) };
  }
  if (exec.startsWith("focus:")) return { kind: "focus_window", id: exec.slice(6) };
  if (exec.startsWith("close-window:")) {
    return { kind: "close_window", id: exec.slice(13) };
  }
  if (exec.startsWith("minimize-window:")) {
    return { kind: "minimize_window", id: exec.slice(16) };
  }
  if (exec.startsWith("move-window-current:")) {
    return { kind: "move_window_current_workspace", id: exec.slice(20) };
  }
  if (exec.startsWith("fill:")) return { kind: "query_fill", value: exec.slice(5) };
  if (exec.startsWith("intent-workflow:")) {
    return {
      kind: "intent_workflow",
      steps: exec
        .slice(16)
        .split("|||")
        .map((s) => s.trim())
        .filter(Boolean),
    };
  }
  if (exec.startsWith("profile-switch:")) {
    return { kind: "profile_switch", id: exec.slice(15) };
  }
  if (exec.startsWith("macro-template:")) {
    return { kind: "macro_template_open", template_id: exec.slice(15) };
  }
  if (exec.startsWith("macro:")) return { kind: "macro_open", id: exec.slice(6) };
  if (exec.startsWith("ext-view:")) {
    const [ext_id, ...rest] = exec.slice(9).split(":");
    return { kind: "extension_view", ext_id, command: rest.join(":") };
  }
  if (exec.startsWith("ext-no-view:")) {
    const [ext_id, ...rest] = exec.slice(12).split(":");
    return { kind: "extension_action", ext_id, command: rest.join(":") };
  }
  if (result.source === "File") return { kind: "open_file", path: exec };
  return { kind: "launch_app", exec };
}

export function commandForResult(result: SearchResult): import("$lib/types").CommandContract {
  return result.command ?? inferCommandFromResult(result);
}

export function execForAction(
  result: SearchResult,
  action: { exec?: string; command?: import("$lib/types").CommandContract },
): string {
  if (action.exec) return action.exec;
  if (action.command) return commandToExec(action.command);
  return result.exec;
}

export function getConfirmSpec(
  result: SearchResult,
): { title: string; description: string; confirmLabel: string } | null {
  const command = commandForResult(result);
  if (command.kind === "intent_workflow") {
    return {
      title: "Run Adaptive Workflow?",
      description: `This will execute ${command.steps.length} inferred intent steps in sequence.`,
      confirmLabel: "Run Workflow",
    };
  }

  if (command.kind === "close_window") {
    return {
      title: "Close Window?",
      description: `This will close "${result.title}" immediately.`,
      confirmLabel: "Close",
    };
  }

  if (command.kind !== "system_action") return null;

  const action = command.action;
  const specs: Record<string, { title: string; description: string; confirmLabel: string }> = {
    shutdown: {
      title: "Shut Down System?",
      description: "This powers off the device immediately.",
      confirmLabel: "Shut Down",
    },
    restart: {
      title: "Restart System?",
      description: "This reboots the device and closes active sessions.",
      confirmLabel: "Restart",
    },
    sleep: {
      title: "Sleep System?",
      description: "This suspends the current session.",
      confirmLabel: "Sleep",
    },
    logout: {
      title: "Log Out Session?",
      description: "This terminates the current user session.",
      confirmLabel: "Log Out",
    },
    bios: {
      title: "Reboot To Firmware Setup?",
      description: "This restarts the machine and enters firmware setup if supported.",
      confirmLabel: "Reboot",
    },
  };

  return specs[action] ?? null;
}

export function normalizeSettingsSection(section: string | null | undefined): string | null {
  if (!section) return null;
  const trimmed = section.trim();
  if (!trimmed) return null;
  const lowered = trimmed.toLowerCase();
  const aliases: Record<string, string> = {
    "theme": "Theme Profile",
    "appearance": "Theme Profile",
    "extensions": "Extensions",
    "window": "Window",
    "general": "General",
    "community": "Community",
    "feature-hub": "Feature Hub",
    "feature hub": "Feature Hub",
    "accessibility": "Accessibility",
    "file-search": "File Search",
    "file search": "File Search",
    "diagnostics": "Diagnostics",
  };
  return aliases[lowered] ?? trimmed;
}

export function parsePermissionError(err: unknown):
  | { type: "needed"; payload: import("$lib/types").PermissionNeededPayload }
  | null {
  const raw =
    typeof err === "string"
      ? err
      : typeof err === "object" && err && "message" in err
        ? String((err as any).message)
        : String(err ?? "");

  if (!raw) return null;

  const neededPrefix = "PERMISSION_NEEDED:";
  if (raw.startsWith(neededPrefix)) {
    try {
      const payload = JSON.parse(raw.slice(neededPrefix.length));
      return { type: "needed", payload } as any;
    } catch (e) {
      console.warn("Failed to parse PERMISSION_NEEDED payload", e, raw);
    }
  }

  return null;
}
