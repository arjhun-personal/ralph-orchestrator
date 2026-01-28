/**
 * HatPalette Component Tests
 *
 * Tests that HatPalette renders hat templates, supports
 * search filtering, and includes the Reroute utility item.
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

// Mock @xyflow/react since it uses DOM APIs not available in jsdom
vi.mock("@xyflow/react", () => ({
  Handle: ({ id, type, ...props }: any) => (
    <div data-testid={`handle-${type}-${id}`} {...props} />
  ),
  Position: { Left: "left", Right: "right" },
}));

import { HatPalette } from "./HatPalette";

describe("HatPalette", () => {
  it("renders all hat template names", () => {
    render(<HatPalette />);

    expect(screen.getByText("Planner")).toBeInTheDocument();
    expect(screen.getByText("Builder")).toBeInTheDocument();
    expect(screen.getByText("Reviewer")).toBeInTheDocument();
    expect(screen.getByText("Validator")).toBeInTheDocument();
    expect(screen.getByText("Confessor")).toBeInTheDocument();
    expect(screen.getByText("Custom Hat")).toBeInTheDocument();
  });

  it("renders search input", () => {
    render(<HatPalette />);

    const searchInput = screen.getByPlaceholderText("Search templates...");
    expect(searchInput).toBeInTheDocument();
  });

  it("filters templates when search query is entered", () => {
    render(<HatPalette />);

    const searchInput = screen.getByPlaceholderText("Search templates...");
    fireEvent.change(searchInput, { target: { value: "plan" } });

    expect(screen.getByText("Planner")).toBeInTheDocument();
    expect(screen.queryByText("Builder")).not.toBeInTheDocument();
    expect(screen.queryByText("Reviewer")).not.toBeInTheDocument();
    expect(screen.queryByText("Validator")).not.toBeInTheDocument();
    expect(screen.queryByText("Confessor")).not.toBeInTheDocument();
    expect(screen.queryByText("Custom Hat")).not.toBeInTheDocument();
  });

  it("shows 'No matching templates' when search has no results", () => {
    render(<HatPalette />);

    const searchInput = screen.getByPlaceholderText("Search templates...");
    fireEvent.change(searchInput, { target: { value: "zzzznonexistent" } });

    expect(screen.getByText("No matching templates")).toBeInTheDocument();
  });

  it("renders the Reroute utility item", () => {
    render(<HatPalette />);

    expect(screen.getByText("Reroute")).toBeInTheDocument();
    expect(screen.getByText("Waypoint for connection routing")).toBeInTheDocument();
  });
});
