import { test } from "node:test";
import assert from "node:assert";
import stripAnsi from "strip-ansi";
import { TaskBridge } from "./TaskBridge";
import { TaskRepository } from "../repositories";
import { TaskQueueService, EventBus } from "../queue";
import { initializeDatabase, getDatabase } from "../db/connection";
import { tasks } from "../db/schema";

test("strip-ansi removes ANSI codes", () => {
  const input = "\u001B[4mHello World\u001B[0m";
  const expected = "Hello World";
  const actual = stripAnsi(input);
  assert.strictEqual(actual, expected);
});

test("strip-ansi handles plain text", () => {
  const input = "Hello World";
  const actual = stripAnsi(input);
  assert.strictEqual(actual, input);
});

test("recoverStuckTasks marks running tasks as failed", async () => {
  // Setup in-memory DB
  initializeDatabase(getDatabase(":memory:"));
  const db = getDatabase();

  // Clean up
  db.delete(tasks).run();

  const taskRepository = new TaskRepository(db);
  const taskQueue = new TaskQueueService();
  const eventBus = new EventBus();

  const taskBridge = new TaskBridge(taskRepository, taskQueue, eventBus, {
    defaultCwd: process.cwd(),
  });

  // Create a stuck task
  const now = new Date();
  const stuckTask = {
    id: "task-stuck",
    title: "Stuck Task",
    status: "running",
    priority: 1,
    createdAt: now,
    updatedAt: now,
    startedAt: now,
  };

  // Manually insert stuck task
  db.insert(tasks).values(stuckTask).run();

  // Create a normal task
  taskRepository.create({
    id: "task-normal",
    title: "Normal Task",
    status: "open",
    priority: 1,
  });

  // Run recovery
  const recoveredCount = taskBridge.recoverStuckTasks();

  // Assertions
  assert.strictEqual(recoveredCount, 1, "Should recover 1 task");

  const updatedStuckTask = taskRepository.findById("task-stuck");
  assert.strictEqual(updatedStuckTask?.status, "failed", "Stuck task should be failed");
  assert.ok(
    updatedStuckTask?.errorMessage?.includes("Server restarted"),
    "Should have error message"
  );

  const normalTask = taskRepository.findById("task-normal");
  assert.strictEqual(normalTask?.status, "open", "Normal task should be untouched");
});
