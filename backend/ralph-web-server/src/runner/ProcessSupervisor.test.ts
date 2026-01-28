import { test } from "node:test";
import assert from "node:assert";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import { ProcessSupervisor } from "./ProcessSupervisor";

const testRunDir = path.join(os.tmpdir(), "ralph-test-runs");

test("ProcessSupervisor.spawn creates task directory and PID file", () => {
  const supervisor = new ProcessSupervisor({ runDir: testRunDir });
  const taskId = "test-spawn-" + Date.now();

  const handle = supervisor.spawn(taskId, "test prompt", ["--version"], process.cwd());

  assert.ok(handle.pid > 0);
  assert.strictEqual(handle.taskId, taskId);
  assert.ok(fs.existsSync(handle.taskDir));
  assert.ok(fs.existsSync(path.join(handle.taskDir, "pid")));
  assert.ok(fs.existsSync(path.join(handle.taskDir, "status.json")));

  // Cleanup
  fs.rmSync(handle.taskDir, { recursive: true, force: true });
});

test("ProcessSupervisor.spawn writes initial status", () => {
  const supervisor = new ProcessSupervisor({ runDir: testRunDir });
  const taskId = "test-status-" + Date.now();

  const handle = supervisor.spawn(taskId, "test prompt", ["--version"], process.cwd());
  const status = supervisor.getStatus(taskId);

  assert.ok(status);
  assert.strictEqual(status.state, "running");
  assert.ok(status.startedAt);

  // Cleanup
  fs.rmSync(handle.taskDir, { recursive: true, force: true });
});

test("ProcessSupervisor.reconnect returns null for non-existent task", () => {
  const supervisor = new ProcessSupervisor({ runDir: testRunDir });
  const handle = supervisor.reconnect("non-existent-task");

  assert.strictEqual(handle, null);
});

test("ProcessSupervisor.reconnect returns handle for existing process", () => {
  const supervisor = new ProcessSupervisor({ runDir: testRunDir });
  const taskId = "test-reconnect-" + Date.now();

  const original = supervisor.spawn(taskId, "test prompt", ["--version"], process.cwd());
  const reconnected = supervisor.reconnect(taskId);

  assert.ok(reconnected);
  assert.strictEqual(reconnected.taskId, taskId);
  assert.strictEqual(reconnected.pid, original.pid);

  // Cleanup
  fs.rmSync(original.taskDir, { recursive: true, force: true });
});

test("ProcessSupervisor.isAlive returns true for current process", () => {
  const supervisor = new ProcessSupervisor({ runDir: testRunDir });
  const alive = supervisor.isAlive(process.pid);

  assert.strictEqual(alive, true);
});

test("ProcessSupervisor.isAlive returns false for non-existent PID", () => {
  const supervisor = new ProcessSupervisor({ runDir: testRunDir });
  const alive = supervisor.isAlive(999999);

  assert.strictEqual(alive, false);
});

test("ProcessSupervisor.getStatus returns null for non-existent task", () => {
  const supervisor = new ProcessSupervisor({ runDir: testRunDir });
  const status = supervisor.getStatus("non-existent-task");

  assert.strictEqual(status, null);
});

test("ProcessSupervisor.getStatus returns status for existing task", () => {
  const supervisor = new ProcessSupervisor({ runDir: testRunDir });
  const taskId = "test-getstatus-" + Date.now();

  const handle = supervisor.spawn(taskId, "test prompt", ["--version"], process.cwd());
  const status = supervisor.getStatus(taskId);

  assert.ok(status);
  assert.strictEqual(status.state, "running");

  // Cleanup
  fs.rmSync(handle.taskDir, { recursive: true, force: true });
});
