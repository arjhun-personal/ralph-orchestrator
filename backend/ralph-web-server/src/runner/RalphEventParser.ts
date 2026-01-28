/**
 * RalphEventParser
 *
 * Parses Ralph orchestrator events from stdout lines.
 * Events are emitted as JSONL with format:
 *   {"ts":"...","iteration":N,"hat":"...","topic":"...","triggered":"...","payload":"..."}
 *
 * The parser detects lines that are valid JSON objects with a 'topic' field
 * and invokes the callback with the parsed event.
 */

/**
 * Parsed Ralph event from stdout
 */
export interface RalphEvent {
  /** ISO timestamp of the event */
  ts: string;
  /** Iteration number (optional) */
  iteration?: number;
  /** Hat that emitted the event (optional) */
  hat?: string;
  /** Event topic (e.g., "build.done", "confession.clean") */
  topic: string;
  /** Event that triggered this one (optional) */
  triggered?: string;
  /** Event payload - can be string, object, or null */
  payload: string | Record<string, unknown> | null;
}

/**
 * Callback invoked when an event is parsed
 */
export type EventCallback = (event: RalphEvent) => void;

/**
 * RalphEventParser
 *
 * Detects and parses JSONL events from stdout lines.
 */
export class RalphEventParser {
  private readonly onEvent: EventCallback;

  constructor(onEvent: EventCallback) {
    this.onEvent = onEvent;
  }

  /**
   * Parse a single line and emit event if valid.
   * Non-event lines are silently ignored.
   */
  parseLine(line: string): void {
    const trimmed = line.trim();

    // Quick check: must start with { and end with }
    if (!trimmed.startsWith("{") || !trimmed.endsWith("}")) {
      return;
    }

    try {
      const parsed = JSON.parse(trimmed);

      // Must have a string 'topic' field to be considered an event
      if (typeof parsed.topic !== "string") {
        return;
      }

      // Construct the event object
      const event: RalphEvent = {
        ts: parsed.ts ?? new Date().toISOString(),
        topic: parsed.topic,
        payload: parsed.payload ?? null,
      };

      // Copy optional fields if present
      if (typeof parsed.iteration === "number") {
        event.iteration = parsed.iteration;
      }
      if (typeof parsed.hat === "string") {
        event.hat = parsed.hat;
      }
      if (typeof parsed.triggered === "string") {
        event.triggered = parsed.triggered;
      }

      this.onEvent(event);
    } catch {
      // Not valid JSON - ignore
    }
  }

  /**
   * Static helper to check if a line looks like an event.
   * Useful for filtering logs before display.
   */
  static isEventLine(line: string): boolean {
    const trimmed = line.trim();
    if (!trimmed.startsWith("{") || !trimmed.endsWith("}")) {
      return false;
    }

    try {
      const parsed = JSON.parse(trimmed);
      return typeof parsed.topic === "string";
    } catch {
      return false;
    }
  }
}
