/**
 * PropertiesPanel Component Tests
 *
 * Tests that PropertiesPanel shows the correct empty state,
 * renders form fields for selected nodes, supports editing,
 * and provides a delete button.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

// Mock @xyflow/react in case it is imported transitively
vi.mock("@xyflow/react", () => ({
  Handle: ({ id, type, ...props }: any) => (
    <div data-testid={`handle-${type}-${id}`} {...props} />
  ),
  Position: { Left: "left", Right: "right" },
}));

import { PropertiesPanel } from "./PropertiesPanel";
import type { HatNodeData } from "./HatNode";

const mockOnUpdateNode = vi.fn();
const mockOnDeleteNode = vi.fn();

function makeSelectedNode(overrides: Partial<HatNodeData> = {}) {
  return {
    id: "node-1",
    data: {
      key: "builder",
      name: "Builder",
      description: "Implements code, runs tests, creates commits",
      triggersOn: ["build.task"],
      publishes: ["build.done"],
      instructions: "Follow TDD",
      ...overrides,
    },
  };
}

describe("PropertiesPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows empty state message when no node is selected", () => {
    render(
      <PropertiesPanel
        selectedNode={null}
        onUpdateNode={mockOnUpdateNode}
        onDeleteNode={mockOnDeleteNode}
      />
    );

    expect(
      screen.getByText("Select a hat on the canvas to edit its properties")
    ).toBeInTheDocument();
  });

  it("renders form fields when a node is selected", () => {
    const node = makeSelectedNode();

    render(
      <PropertiesPanel
        selectedNode={node}
        onUpdateNode={mockOnUpdateNode}
        onDeleteNode={mockOnDeleteNode}
      />
    );

    // Key field (disabled input with value)
    const keyInput = screen.getByDisplayValue("builder");
    expect(keyInput).toBeInTheDocument();
    expect(keyInput).toBeDisabled();

    // Name field
    const nameInput = screen.getByDisplayValue("Builder");
    expect(nameInput).toBeInTheDocument();

    // Description field
    const descriptionField = screen.getByDisplayValue(
      "Implements code, runs tests, creates commits"
    );
    expect(descriptionField).toBeInTheDocument();

    // Labels
    expect(screen.getByText("Key")).toBeInTheDocument();
    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("Description")).toBeInTheDocument();
    expect(screen.getByText("Triggers On (Inputs)")).toBeInTheDocument();
    expect(screen.getByText("Publishes (Outputs)")).toBeInTheDocument();
    expect(screen.getByText("Instructions")).toBeInTheDocument();
  });

  it("shows 'Delete Hat' button when a node is selected", () => {
    const node = makeSelectedNode();

    render(
      <PropertiesPanel
        selectedNode={node}
        onUpdateNode={mockOnUpdateNode}
        onDeleteNode={mockOnDeleteNode}
      />
    );

    expect(screen.getByRole("button", { name: /delete hat/i })).toBeInTheDocument();
  });

  it("calls onUpdateNode when name field is changed", () => {
    const node = makeSelectedNode();

    render(
      <PropertiesPanel
        selectedNode={node}
        onUpdateNode={mockOnUpdateNode}
        onDeleteNode={mockOnDeleteNode}
      />
    );

    const nameInput = screen.getByDisplayValue("Builder");
    fireEvent.change(nameInput, { target: { value: "Super Builder" } });

    expect(mockOnUpdateNode).toHaveBeenCalledWith("node-1", {
      name: "Super Builder",
    });
  });

  it("shows 'Properties' title", () => {
    render(
      <PropertiesPanel
        selectedNode={null}
        onUpdateNode={mockOnUpdateNode}
        onDeleteNode={mockOnDeleteNode}
      />
    );

    expect(screen.getByText("Properties")).toBeInTheDocument();
  });
});
