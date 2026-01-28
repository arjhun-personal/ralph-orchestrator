/**
 * Task Router Tests - Extended Fields
 *
 * Tests for Step 9 of Task UX Improvements:
 * - preset: Hat collection/preset used
 * - currentIteration: Current iteration count
 * - maxIterations: Max iterations configured
 * - loopId: Associated loop ID
 *
 * These fields enable rich task detail display in the UI.
 */

import { describe, it, beforeEach } from "node:test";
import assert from "node:assert/strict";
import { TaskRepository } from "../repositories";
import { initializeDatabase, getDatabase } from "../db/connection";
import { tasks } from "../db/schema";

describe("task schema supports new fields for UX improvements", () => {
  let taskRepository: TaskRepository;

  beforeEach(() => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    db.delete(tasks).run();
    taskRepository = new TaskRepository(db);
  });

  it("preset field can be stored and retrieved", () => {
    const task = taskRepository.create({
      id: "task-preset-test",
      title: "Test task with preset",
      status: "open",
      priority: 2,
      preset: "feature-dev",
    });

    assert.equal(task.preset, "feature-dev");

    const retrieved = taskRepository.findById("task-preset-test");
    assert.equal(retrieved?.preset, "feature-dev");
  });

  it("currentIteration and maxIterations fields can be stored", () => {
    const task = taskRepository.create({
      id: "task-iteration-test",
      title: "Test task with iteration",
      status: "running",
      priority: 2,
      currentIteration: 5,
      maxIterations: 50,
    });

    assert.equal(task.currentIteration, 5);
    assert.equal(task.maxIterations, 50);

    const retrieved = taskRepository.findById("task-iteration-test");
    assert.equal(retrieved?.currentIteration, 5);
    assert.equal(retrieved?.maxIterations, 50);
  });

  it("loopId field can be stored and retrieved", () => {
    const task = taskRepository.create({
      id: "task-loop-test",
      title: "Test task with loop association",
      status: "running",
      priority: 2,
      loopId: "ralph-20260128-041234-abc1",
    });

    assert.equal(task.loopId, "ralph-20260128-041234-abc1");

    const retrieved = taskRepository.findById("task-loop-test");
    assert.equal(retrieved?.loopId, "ralph-20260128-041234-abc1");
  });

  it("iteration fields can be updated during execution", () => {
    taskRepository.create({
      id: "task-update-iteration",
      title: "Task that will have iteration updated",
      status: "running",
      priority: 2,
    });

    const updated = taskRepository.update("task-update-iteration", {
      currentIteration: 12,
      maxIterations: 30,
    });

    assert.equal(updated?.currentIteration, 12);
    assert.equal(updated?.maxIterations, 30);
  });

  it("all new fields can be set together", () => {
    const task = taskRepository.create({
      id: "task-all-fields",
      title: "Task with all new fields",
      status: "running",
      priority: 2,
      preset: "tdd-workflow",
      currentIteration: 7,
      maxIterations: 25,
      loopId: "ralph-20260128-043000-xyz9",
    });

    assert.equal(task.preset, "tdd-workflow");
    assert.equal(task.currentIteration, 7);
    assert.equal(task.maxIterations, 25);
    assert.equal(task.loopId, "ralph-20260128-043000-xyz9");
  });

  it("new fields are nullable (backward compatible)", () => {
    const task = taskRepository.create({
      id: "task-nullable",
      title: "Task without new fields",
      status: "open",
      priority: 2,
    });

    assert.equal(task.preset, null);
    assert.equal(task.currentIteration, null);
    assert.equal(task.maxIterations, null);
    assert.equal(task.loopId, null);
  });
});

describe("task list and get endpoints return new fields", () => {
  let taskRepository: TaskRepository;

  beforeEach(() => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    db.delete(tasks).run();
    taskRepository = new TaskRepository(db);
  });

  it("findAll returns tasks with new fields", () => {
    taskRepository.create({
      id: "task-list-1",
      title: "Task 1",
      status: "running",
      priority: 2,
      preset: "preset-a",
      currentIteration: 3,
      maxIterations: 10,
      loopId: "loop-abc",
    });

    taskRepository.create({
      id: "task-list-2",
      title: "Task 2",
      status: "open",
      priority: 2,
    });

    const allTasks = taskRepository.findAll();

    const task1 = allTasks.find((t) => t.id === "task-list-1");
    assert.ok(task1);
    assert.equal(task1?.preset, "preset-a");
    assert.equal(task1?.currentIteration, 3);
    assert.equal(task1?.maxIterations, 10);
    assert.equal(task1?.loopId, "loop-abc");

    const task2 = allTasks.find((t) => t.id === "task-list-2");
    assert.ok(task2);
    assert.equal(task2?.preset, null);
    assert.equal(task2?.currentIteration, null);
  });

  it("findById returns task with new fields", () => {
    taskRepository.create({
      id: "task-find-test",
      title: "Task for findById",
      status: "running",
      priority: 2,
      preset: "preset-b",
      currentIteration: 5,
      maxIterations: 15,
      loopId: "loop-xyz",
    });

    const task = taskRepository.findById("task-find-test");

    assert.ok(task);
    assert.equal(task?.preset, "preset-b");
    assert.equal(task?.currentIteration, 5);
    assert.equal(task?.maxIterations, 15);
    assert.equal(task?.loopId, "loop-xyz");
  });
});
