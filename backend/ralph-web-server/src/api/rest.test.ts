/**
 * REST API Router Tests
 *
 * Tests for REST endpoints at /api/v1/* using Fastify's inject() method.
 * Covers health, tasks CRUD, task run, hats, and presets endpoints.
 */

import { describe, it, beforeEach } from "node:test";
import assert from "node:assert/strict";
import { FastifyInstance } from "fastify";
import { TaskRepository } from "../repositories";
import { initializeDatabase, getDatabase } from "../db/connection";
import { tasks } from "../db/schema";
import { createServer } from "./server";

describe("GET /api/v1/health", () => {
  let server: FastifyInstance;

  beforeEach(async () => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    server = await createServer({ db, logger: false });
    await server.ready();
  });

  it("returns 200 with status, version, and timestamp", async () => {
    const response = await server.inject({
      method: "GET",
      url: "/api/v1/health",
    });

    assert.equal(response.statusCode, 200);
    const body = JSON.parse(response.body);
    assert.equal(body.status, "ok");
    assert.equal(body.version, "1.0.0");
    assert.ok(body.timestamp, "should have a timestamp");
    // Verify timestamp is a valid ISO string
    assert.ok(!isNaN(Date.parse(body.timestamp)), "timestamp should be valid ISO date");
  });
});

describe("GET /api/v1/tasks", () => {
  let server: FastifyInstance;
  let taskRepository: TaskRepository;

  beforeEach(async () => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    db.delete(tasks).run();
    taskRepository = new TaskRepository(db);
    server = await createServer({ db, logger: false });
    await server.ready();
  });

  it("returns task list", async () => {
    taskRepository.create({ id: "t1", title: "Task 1", status: "open", priority: 2 });
    taskRepository.create({ id: "t2", title: "Task 2", status: "running", priority: 1 });

    const response = await server.inject({
      method: "GET",
      url: "/api/v1/tasks",
    });

    assert.equal(response.statusCode, 200);
    const body = JSON.parse(response.body);
    assert.ok(Array.isArray(body), "response should be an array");
    assert.equal(body.length, 2);
  });

  it("filters by status query parameter", async () => {
    taskRepository.create({ id: "t1", title: "Open Task", status: "open", priority: 2 });
    taskRepository.create({ id: "t2", title: "Running Task", status: "running", priority: 1 });
    taskRepository.create({ id: "t3", title: "Another Open", status: "open", priority: 3 });

    const response = await server.inject({
      method: "GET",
      url: "/api/v1/tasks?status=open",
    });

    assert.equal(response.statusCode, 200);
    const body = JSON.parse(response.body);
    assert.equal(body.length, 2);
    for (const task of body) {
      assert.equal(task.status, "open");
    }
  });
});

describe("POST /api/v1/tasks", () => {
  let server: FastifyInstance;

  beforeEach(async () => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    db.delete(tasks).run();
    server = await createServer({ db, logger: false });
    await server.ready();
  });

  it("creates a task and returns 201", async () => {
    const response = await server.inject({
      method: "POST",
      url: "/api/v1/tasks",
      headers: { "content-type": "application/json" },
      payload: { id: "new-task", title: "My New Task" },
    });

    assert.equal(response.statusCode, 201);
    const body = JSON.parse(response.body);
    assert.equal(body.id, "new-task");
    assert.equal(body.title, "My New Task");
    assert.equal(body.status, "open");
    assert.equal(body.priority, 2);
  });

  it("returns 400 when missing required fields", async () => {
    const response = await server.inject({
      method: "POST",
      url: "/api/v1/tasks",
      headers: { "content-type": "application/json" },
      payload: { title: "No ID provided" },
    });

    assert.equal(response.statusCode, 400);
    const body = JSON.parse(response.body);
    assert.equal(body.error, "Bad Request");
    assert.ok(body.message.includes("id and title are required"));
  });

  it("returns 400 when priority is out of range", async () => {
    const response = await server.inject({
      method: "POST",
      url: "/api/v1/tasks",
      headers: { "content-type": "application/json" },
      payload: { id: "bad-priority", title: "Bad Priority", priority: 10 },
    });

    assert.equal(response.statusCode, 400);
    const body = JSON.parse(response.body);
    assert.equal(body.error, "Bad Request");
    assert.ok(body.message.includes("priority must be between 1 and 5"));
  });
});

describe("GET /api/v1/tasks/:id", () => {
  let server: FastifyInstance;
  let taskRepository: TaskRepository;

  beforeEach(async () => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    db.delete(tasks).run();
    taskRepository = new TaskRepository(db);
    server = await createServer({ db, logger: false });
    await server.ready();
  });

  it("returns task by ID", async () => {
    taskRepository.create({ id: "find-me", title: "Find Me", status: "open", priority: 3 });

    const response = await server.inject({
      method: "GET",
      url: "/api/v1/tasks/find-me",
    });

    assert.equal(response.statusCode, 200);
    const body = JSON.parse(response.body);
    assert.equal(body.id, "find-me");
    assert.equal(body.title, "Find Me");
    assert.equal(body.priority, 3);
  });

  it("returns 404 for missing task", async () => {
    const response = await server.inject({
      method: "GET",
      url: "/api/v1/tasks/does-not-exist",
    });

    assert.equal(response.statusCode, 404);
    const body = JSON.parse(response.body);
    assert.equal(body.error, "Not Found");
    assert.ok(body.message.includes("does-not-exist"));
  });
});

describe("PATCH /api/v1/tasks/:id", () => {
  let server: FastifyInstance;
  let taskRepository: TaskRepository;

  beforeEach(async () => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    db.delete(tasks).run();
    taskRepository = new TaskRepository(db);
    server = await createServer({ db, logger: false });
    await server.ready();
  });

  it("updates fields and returns updated task", async () => {
    taskRepository.create({ id: "update-me", title: "Original", status: "open", priority: 2 });

    const response = await server.inject({
      method: "PATCH",
      url: "/api/v1/tasks/update-me",
      headers: { "content-type": "application/json" },
      payload: { title: "Updated Title", priority: 4 },
    });

    assert.equal(response.statusCode, 200);
    const body = JSON.parse(response.body);
    assert.equal(body.id, "update-me");
    assert.equal(body.title, "Updated Title");
    assert.equal(body.priority, 4);
  });

  it("returns 400 for empty title", async () => {
    taskRepository.create({ id: "empty-title", title: "Has Title", status: "open", priority: 2 });

    const response = await server.inject({
      method: "PATCH",
      url: "/api/v1/tasks/empty-title",
      headers: { "content-type": "application/json" },
      payload: { title: "" },
    });

    assert.equal(response.statusCode, 400);
    const body = JSON.parse(response.body);
    assert.equal(body.error, "Bad Request");
    assert.ok(body.message.includes("title must not be empty"));
  });

  it("returns 404 for missing task", async () => {
    const response = await server.inject({
      method: "PATCH",
      url: "/api/v1/tasks/ghost",
      headers: { "content-type": "application/json" },
      payload: { title: "Does not matter" },
    });

    assert.equal(response.statusCode, 404);
    const body = JSON.parse(response.body);
    assert.equal(body.error, "Not Found");
  });
});

describe("DELETE /api/v1/tasks/:id", () => {
  let server: FastifyInstance;
  let taskRepository: TaskRepository;

  beforeEach(async () => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    db.delete(tasks).run();
    taskRepository = new TaskRepository(db);
    server = await createServer({ db, logger: false });
    await server.ready();
  });

  it("returns 204 for task in terminal state", async () => {
    taskRepository.create({ id: "del-me", title: "Failed Task", status: "failed", priority: 2 });

    const response = await server.inject({
      method: "DELETE",
      url: "/api/v1/tasks/del-me",
    });

    assert.equal(response.statusCode, 204);
  });

  it("returns 404 for missing task", async () => {
    const response = await server.inject({
      method: "DELETE",
      url: "/api/v1/tasks/no-such-task",
    });

    assert.equal(response.statusCode, 404);
    const body = JSON.parse(response.body);
    assert.equal(body.error, "Not Found");
  });

  it("returns 409 for task in active state", async () => {
    taskRepository.create({ id: "running-task", title: "Running", status: "running", priority: 2 });

    const response = await server.inject({
      method: "DELETE",
      url: "/api/v1/tasks/running-task",
    });

    assert.equal(response.statusCode, 409);
    const body = JSON.parse(response.body);
    assert.equal(body.error, "Conflict");
    assert.ok(body.message.includes("running"));
  });
});

describe("POST /api/v1/tasks/:id/run", () => {
  it("returns 503 when no taskBridge configured", async () => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    db.delete(tasks).run();
    const server = await createServer({ db, logger: false });
    await server.ready();

    const taskRepository = new TaskRepository(db);
    taskRepository.create({ id: "run-503", title: "No Bridge", status: "open", priority: 2 });

    const response = await server.inject({
      method: "POST",
      url: "/api/v1/tasks/run-503/run",
    });

    assert.equal(response.statusCode, 503);
    const body = JSON.parse(response.body);
    assert.equal(body.error, "Service Unavailable");
  });

  it("returns 404 for missing task with taskBridge", async () => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    db.delete(tasks).run();

    const mockBridge = {
      enqueueTask: () => ({ success: true, queuedTaskId: "q1" }),
    } as any;

    const server = await createServer({ db, logger: false, taskBridge: mockBridge });
    await server.ready();

    const response = await server.inject({
      method: "POST",
      url: "/api/v1/tasks/nonexistent/run",
    });

    assert.equal(response.statusCode, 404);
    const body = JSON.parse(response.body);
    assert.equal(body.error, "Not Found");
    assert.ok(body.message.includes("nonexistent"));
  });
});

describe("GET /api/v1/hats", () => {
  let server: FastifyInstance;

  beforeEach(async () => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    server = await createServer({ db, logger: false });
    await server.ready();
  });

  it("returns hat list as array", async () => {
    const response = await server.inject({
      method: "GET",
      url: "/api/v1/hats",
    });

    assert.equal(response.statusCode, 200);
    const body = JSON.parse(response.body);
    assert.ok(Array.isArray(body), "response should be an array");
  });
});

describe("GET /api/v1/presets", () => {
  let server: FastifyInstance;

  beforeEach(async () => {
    initializeDatabase(getDatabase(":memory:"));
    const db = getDatabase();
    server = await createServer({ db, logger: false });
    await server.ready();
  });

  it("returns preset list as array", async () => {
    const response = await server.inject({
      method: "GET",
      url: "/api/v1/presets",
    });

    assert.equal(response.statusCode, 200);
    const body = JSON.parse(response.body);
    assert.ok(Array.isArray(body), "response should be an array");
  });
});
