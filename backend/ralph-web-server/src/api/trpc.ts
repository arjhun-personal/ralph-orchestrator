/**
 * TRPC Router Configuration
 *
 * Defines the TRPC router with task-related procedures.
 * Uses the existing TaskRepository for data access.
 */

import { initTRPC, TRPCError } from "@trpc/server";
import { z } from "zod";
import { TaskRepository, SettingsRepository, TaskLogRepository, CollectionRepository } from "../repositories";
import { SettingsService } from "../services/SettingsService";
import { TaskBridge } from "../services/TaskBridge";
import { LoopsManager } from "../services/LoopsManager";
import { PlanningService } from "../services/PlanningService";
import { CollectionService } from "../services/CollectionService";
import { BetterSQLite3Database } from "drizzle-orm/better-sqlite3";
import * as schema from "../db/schema";

/**
 * Context passed to all TRPC procedures
 */
export interface Context {
  taskRepository: TaskRepository;
  taskLogRepository: TaskLogRepository;
  settingsService: SettingsService;
  collectionService: CollectionService;
  taskBridge?: TaskBridge;
  loopsManager?: LoopsManager;
  planningService?: PlanningService;
}

/**
 * Create context from database instance
 * @param db - Database instance
 * @param taskBridge - Optional TaskBridge for task execution
 * @param loopsManager - Optional LoopsManager for loop operations
 * @param planningService - Optional PlanningService for planning sessions
 */
export function createContext(
  db: BetterSQLite3Database<typeof schema>,
  taskBridge?: TaskBridge,
  loopsManager?: LoopsManager,
  planningService?: PlanningService
): Context {
  const settingsRepository = new SettingsRepository(db);
  const collectionRepository = new CollectionRepository(db);
  return {
    taskRepository: new TaskRepository(db),
    taskLogRepository: new TaskLogRepository(db),
    settingsService: new SettingsService(settingsRepository),
    collectionService: new CollectionService(collectionRepository),
    taskBridge,
    loopsManager,
    planningService,
  };
}

const t = initTRPC.context<Context>().create();

export const router = t.router;
export const publicProcedure = t.procedure;

/**
 * Task router - CRUD operations for tasks
 */
export const taskRouter = router({
  /**
   * List all tasks, optionally filtered by status and archival state
   */
  list: publicProcedure
    .input(
      z
        .object({
          status: z.string().optional(),
          includeArchived: z.boolean().default(false).optional(),
        })
        .optional()
    )
    .query(({ ctx, input }) => {
      return ctx.taskRepository.findAll(input?.status, input?.includeArchived);
    }),

  /**
   * Get a single task by ID
   */
  get: publicProcedure.input(z.object({ id: z.string() })).query(({ ctx, input }) => {
    const task = ctx.taskRepository.findById(input.id);
    if (!task) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Task with id '${input.id}' not found`,
      });
    }
    return task;
  }),

  /**
   * Get tasks that are ready to be worked on (not blocked)
   */
  ready: publicProcedure.query(({ ctx }) => {
    return ctx.taskRepository.findReady();
  }),

  /**
   * Create a new task and auto-execute it
   */
  create: publicProcedure
    .input(
      z.object({
        id: z.string(),
        title: z.string().min(1),
        status: z.string().default("open"),
        priority: z.number().int().min(1).max(5).default(2),
        blockedBy: z.string().nullable().optional(),
        autoExecute: z.boolean().default(true),
      })
    )
    .mutation(({ ctx, input }) => {
      const { autoExecute, ...taskData } = input;
      const task = ctx.taskRepository.create(taskData);

      // Auto-execute the task if requested and bridge is available
      if (autoExecute && ctx.taskBridge && !task.blockedBy) {
        ctx.taskBridge.enqueueTask(task);
        // Return the updated task with pending status
        return ctx.taskRepository.findById(task.id) ?? task;
      }

      return task;
    }),

  /**
   * Run a specific task (enqueue for execution)
   */
  run: publicProcedure.input(z.object({ id: z.string() })).mutation(({ ctx, input }) => {
    if (!ctx.taskBridge) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Task execution is not configured",
      });
    }

    const task = ctx.taskRepository.findById(input.id);
    if (!task) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Task with id '${input.id}' not found`,
      });
    }

    const result = ctx.taskBridge.enqueueTask(task);
    if (!result.success) {
      throw new TRPCError({
        code: "BAD_REQUEST",
        message: result.error || "Failed to enqueue task",
      });
    }

    return {
      success: true,
      queuedTaskId: result.queuedTaskId,
      task: ctx.taskRepository.findById(input.id),
    };
  }),

  /**
   * Run all pending tasks
   */
  runAll: publicProcedure.mutation(({ ctx }) => {
    if (!ctx.taskBridge) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Task execution is not configured",
      });
    }

    const result = ctx.taskBridge.enqueueAllPending();
    return {
      enqueued: result.enqueued,
      errors: result.errors,
    };
  }),

  /**
   * Retry a failed task
   */
  retry: publicProcedure.input(z.object({ id: z.string() })).mutation(({ ctx, input }) => {
    if (!ctx.taskBridge) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Task execution is not configured",
      });
    }

    const result = ctx.taskBridge.retryTask(input.id);
    if (!result.success) {
      throw new TRPCError({
        code: "BAD_REQUEST",
        message: result.error || "Failed to retry task",
      });
    }

    return {
      success: true,
      queuedTaskId: result.queuedTaskId,
      task: ctx.taskRepository.findById(input.id),
    };
  }),

  /**
   * Get execution status for a task
   */
  executionStatus: publicProcedure.input(z.object({ id: z.string() })).query(({ ctx, input }) => {
    if (!ctx.taskBridge) {
      return { isQueued: false };
    }

    return ctx.taskBridge.getExecutionStatus(input.id);
  }),

  /**
   * Cancel a running task
   */
  cancel: publicProcedure.input(z.object({ id: z.string() })).mutation(({ ctx, input }) => {
    if (!ctx.taskBridge) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Task execution is not configured",
      });
    }

    const result = ctx.taskBridge.cancelTask(input.id);
    if (!result.success) {
      throw new TRPCError({
        code: "BAD_REQUEST",
        message: result.error || "Failed to cancel task",
      });
    }

    return {
      success: true,
      task: ctx.taskRepository.findById(input.id),
    };
  }),

  /**
   * Update an existing task
   */
  update: publicProcedure
    .input(
      z.object({
        id: z.string(),
        title: z.string().min(1).optional(),
        status: z.string().optional(),
        priority: z.number().int().min(1).max(5).optional(),
        blockedBy: z.string().nullable().optional(),
      })
    )
    .mutation(({ ctx, input }) => {
      const { id, ...updates } = input;
      const task = ctx.taskRepository.update(id, updates);
      if (!task) {
        throw new TRPCError({
          code: "NOT_FOUND",
          message: `Task with id '${id}' not found`,
        });
      }
      return task;
    }),

  /**
   * Close a task
   */
  close: publicProcedure.input(z.object({ id: z.string() })).mutation(({ ctx, input }) => {
    const task = ctx.taskRepository.close(input.id);
    if (!task) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Task with id '${input.id}' not found`,
      });
    }
    return task;
  }),

  /**
   * Archive a task
   */
  archive: publicProcedure.input(z.object({ id: z.string() })).mutation(({ ctx, input }) => {
    const task = ctx.taskRepository.archive(input.id);
    if (!task) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Task with id '${input.id}' not found`,
      });
    }
    return task;
  }),

  /**
   * Unarchive a task
   */
  unarchive: publicProcedure.input(z.object({ id: z.string() })).mutation(({ ctx, input }) => {
    const task = ctx.taskRepository.unarchive(input.id);
    if (!task) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Task with id '${input.id}' not found`,
      });
    }
    return task;
  }),

  /**
   * Delete a task
   */
  delete: publicProcedure.input(z.object({ id: z.string() })).mutation(({ ctx, input }) => {
    const deleted = ctx.taskRepository.delete(input.id);
    if (!deleted) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Task with id '${input.id}' not found`,
      });
    }
    return { success: true };
  }),

  /**
   * Delete all tasks and task logs.
   */
  clearAll: publicProcedure.mutation(({ ctx }) => {
    const deletedLogs = ctx.taskLogRepository.deleteAll();
    const deletedTasks = ctx.taskRepository.deleteAll();
    return { success: true, deletedTasks, deletedLogs };
  }),
});

/**
 * Hat router - operations for managing hats (operational roles)
 */
export const hatRouter = router({
  /**
   * List all hat definitions from settings
   */
  list: publicProcedure.query(({ ctx }) => {
    const definitions = ctx.settingsService.getHatDefinitions();
    const activeHat = ctx.settingsService.getActiveHat();

    // Convert map to array with active status
    return Object.entries(definitions).map(([key, hat]) => ({
      key,
      ...hat,
      isActive: key === activeHat,
    }));
  }),

  /**
   * Get the currently active hat
   */
  getActive: publicProcedure.query(({ ctx }) => {
    const activeKey = ctx.settingsService.getActiveHat();
    const definition = ctx.settingsService.getActiveHatDefinition();

    return {
      key: activeKey,
      definition: definition ?? null,
    };
  }),

  /**
   * Get a specific hat by key
   */
  get: publicProcedure.input(z.object({ key: z.string() })).query(({ ctx, input }) => {
    const hat = ctx.settingsService.getHat(input.key);
    if (!hat) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Hat '${input.key}' not found`,
      });
    }
    const activeKey = ctx.settingsService.getActiveHat();
    return {
      key: input.key,
      ...hat,
      isActive: input.key === activeKey,
    };
  }),

  /**
   * Set the active hat
   */
  setActive: publicProcedure.input(z.object({ key: z.string() })).mutation(({ ctx, input }) => {
    const hat = ctx.settingsService.getHat(input.key);
    if (!hat) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Hat '${input.key}' not found`,
      });
    }
    ctx.settingsService.setActiveHat(input.key);
    return { success: true, activeHat: input.key };
  }),

  /**
   * Save (create or update) a hat
   */
  save: publicProcedure
    .input(
      z.object({
        key: z.string().min(1),
        name: z.string().min(1),
        description: z.string(),
        triggersOn: z.array(z.string()),
        publishes: z.array(z.string()),
        instructions: z.string().optional(),
      })
    )
    .mutation(({ ctx, input }) => {
      const { key, ...definition } = input;
      ctx.settingsService.setHat(key, definition);
      return { success: true, key };
    }),

  /**
   * Delete a hat
   */
  delete: publicProcedure.input(z.object({ key: z.string() })).mutation(({ ctx, input }) => {
    const deleted = ctx.settingsService.deleteHat(input.key);
    if (!deleted) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Hat '${input.key}' not found`,
      });
    }
    return { success: true };
  }),
});

/**
 * Loops router - operations for managing ralph loops
 */
export const loopsRouter = router({
  /**
   * List all loops, optionally including terminal states
   */
  list: publicProcedure
    .input(
      z
        .object({
          includeTerminal: z.boolean().default(false).optional(),
        })
        .optional()
    )
    .query(async ({ ctx, input }) => {
      if (!ctx.loopsManager) {
        throw new TRPCError({
          code: "INTERNAL_SERVER_ERROR",
          message: "LoopsManager is not configured",
        });
      }

      const loops = await ctx.loopsManager.listLoops();

      // Filter out terminal states unless requested
      if (!input?.includeTerminal) {
        return loops.filter(
          (loop) => !["merged", "discarded"].includes(loop.status)
        );
      }

      return loops;
    }),

  /**
   * Get manager status (running state, interval, last processed time)
   */
  managerStatus: publicProcedure.query(({ ctx }) => {
    if (!ctx.loopsManager) {
      return { running: false, intervalMs: 0 };
    }
    return ctx.loopsManager.getStatus();
  }),

  /**
   * Process the merge queue
   */
  process: publicProcedure.mutation(async ({ ctx }) => {
    if (!ctx.loopsManager) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "LoopsManager is not configured",
      });
    }

    await ctx.loopsManager.processMergeQueue();
    return { success: true };
  }),

  /**
   * Prune stale loops from crashed processes
   */
  prune: publicProcedure.mutation(async ({ ctx }) => {
    if (!ctx.loopsManager) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "LoopsManager is not configured",
      });
    }

    await ctx.loopsManager.pruneStale();
    return { success: true };
  }),

  /**
   * Retry a failed merge
   */
  retry: publicProcedure
    .input(z.object({ id: z.string() }))
    .mutation(async ({ ctx, input }) => {
      if (!ctx.loopsManager) {
        throw new TRPCError({
          code: "INTERNAL_SERVER_ERROR",
          message: "LoopsManager is not configured",
        });
      }

      await ctx.loopsManager.retryMerge(input.id);
      return { success: true };
    }),

  /**
   * Discard a stuck loop
   */
  discard: publicProcedure
    .input(z.object({ id: z.string() }))
    .mutation(async ({ ctx, input }) => {
      if (!ctx.loopsManager) {
        throw new TRPCError({
          code: "INTERNAL_SERVER_ERROR",
          message: "LoopsManager is not configured",
        });
      }

      await ctx.loopsManager.discardLoop(input.id);
      return { success: true };
    }),

  /**
   * Stop a running loop
   */
  stop: publicProcedure
    .input(z.object({ id: z.string(), force: z.boolean().optional() }))
    .mutation(async ({ ctx, input }) => {
      if (!ctx.loopsManager) {
        throw new TRPCError({
          code: "INTERNAL_SERVER_ERROR",
          message: "LoopsManager is not configured",
        });
      }

      await ctx.loopsManager.stopLoop(input.id, input.force);
      return { success: true };
    }),

  /**
   * Force merge a loop
   */
  merge: publicProcedure
    .input(z.object({ id: z.string(), force: z.boolean().optional() }))
    .mutation(async ({ ctx, input }) => {
      if (!ctx.loopsManager) {
        throw new TRPCError({
          code: "INTERNAL_SERVER_ERROR",
          message: "LoopsManager is not configured",
        });
      }

      await ctx.loopsManager.mergeLoop(input.id, input.force);
      return { success: true };
    }),
});

/**
 * Zod schema for graph node position
 */
const nodePositionSchema = z.object({
  x: z.number(),
  y: z.number(),
});

/**
 * Zod schema for hat node data
 */
const hatNodeDataSchema = z.object({
  key: z.string(),
  name: z.string(),
  description: z.string(),
  triggersOn: z.array(z.string()),
  publishes: z.array(z.string()),
  instructions: z.string().optional(),
});

/**
 * Zod schema for graph node
 */
const graphNodeSchema = z.object({
  id: z.string(),
  type: z.string(),
  position: nodePositionSchema,
  data: hatNodeDataSchema,
});

/**
 * Zod schema for graph edge
 */
const graphEdgeSchema = z.object({
  id: z.string(),
  source: z.string(),
  target: z.string(),
  sourceHandle: z.string().optional(),
  targetHandle: z.string().optional(),
  label: z.string().optional(),
});

/**
 * Zod schema for viewport
 */
const viewportSchema = z.object({
  x: z.number(),
  y: z.number(),
  zoom: z.number(),
});

/**
 * Zod schema for complete graph data
 */
const graphDataSchema = z.object({
  nodes: z.array(graphNodeSchema),
  edges: z.array(graphEdgeSchema),
  viewport: viewportSchema,
});

/**
 * Collection router - operations for managing hat collections (visual workflow builder)
 */
export const collectionRouter = router({
  /**
   * List all collections (metadata only, no graph data)
   */
  list: publicProcedure.query(({ ctx }) => {
    return ctx.collectionService.listCollections();
  }),

  /**
   * Get a single collection with full graph data
   */
  get: publicProcedure.input(z.object({ id: z.string() })).query(({ ctx, input }) => {
    const collection = ctx.collectionService.getCollection(input.id);
    if (!collection) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Collection with id '${input.id}' not found`,
      });
    }
    return collection;
  }),

  /**
   * Create a new collection
   */
  create: publicProcedure
    .input(
      z.object({
        name: z.string().min(1),
        description: z.string().optional(),
        graph: graphDataSchema.optional(),
      })
    )
    .mutation(({ ctx, input }) => {
      return ctx.collectionService.createCollection(input);
    }),

  /**
   * Update an existing collection
   */
  update: publicProcedure
    .input(
      z.object({
        id: z.string(),
        name: z.string().min(1).optional(),
        description: z.string().optional(),
        graph: graphDataSchema.optional(),
      })
    )
    .mutation(({ ctx, input }) => {
      const { id, ...updates } = input;
      const collection = ctx.collectionService.updateCollection(id, updates);
      if (!collection) {
        throw new TRPCError({
          code: "NOT_FOUND",
          message: `Collection with id '${id}' not found`,
        });
      }
      return collection;
    }),

  /**
   * Delete a collection
   */
  delete: publicProcedure.input(z.object({ id: z.string() })).mutation(({ ctx, input }) => {
    const deleted = ctx.collectionService.deleteCollection(input.id);
    if (!deleted) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Collection with id '${input.id}' not found`,
      });
    }
    return { success: true };
  }),

  /**
   * Export a collection to Ralph YAML preset format
   */
  exportYaml: publicProcedure.input(z.object({ id: z.string() })).query(({ ctx, input }) => {
    const yaml = ctx.collectionService.exportToYaml(input.id);
    if (!yaml) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: `Collection with id '${input.id}' not found`,
      });
    }
    return { yaml };
  }),

  /**
   * Import a YAML preset as a new collection
   */
  importYaml: publicProcedure
    .input(
      z.object({
        yaml: z.string(),
        name: z.string().min(1),
        description: z.string().optional(),
      })
    )
    .mutation(({ ctx, input }) => {
      try {
        return ctx.collectionService.importFromYaml(input.yaml, input.name, input.description);
      } catch (error) {
        throw new TRPCError({
          code: "BAD_REQUEST",
          message: `Failed to import YAML: ${error instanceof Error ? error.message : "Unknown error"}`,
        });
      }
    }),
});

/**
 * Main app router combining all sub-routers
 */
export const appRouter = router({
  task: taskRouter,
  hat: hatRouter,
  loops: loopsRouter,
  collection: collectionRouter,
  planning: router({
    /**
     * List all planning sessions.
     */
    list: publicProcedure.query(async ({ ctx }) => {
      if (!ctx.planningService) {
        throw new TRPCError({
          code: "INTERNAL_SERVER_ERROR",
          message: "PlanningService is not configured",
        });
      }
      return ctx.planningService.listSessions();
    }),

    /**
     * Get a specific planning session with conversation history.
     */
    get: publicProcedure
      .input(z.object({ id: z.string() }))
      .query(async ({ input, ctx }) => {
        if (!ctx.planningService) {
          throw new TRPCError({
            code: "INTERNAL_SERVER_ERROR",
            message: "PlanningService is not configured",
          });
        }
        return ctx.planningService.getSession(input.id);
      }),

    /**
     * Start a new planning session.
     */
    start: publicProcedure
      .input(z.object({ prompt: z.string().min(1) }))
      .mutation(async ({ input, ctx }) => {
        if (!ctx.planningService) {
          throw new TRPCError({
            code: "INTERNAL_SERVER_ERROR",
            message: "PlanningService is not configured",
          });
        }
        return ctx.planningService.startSession(input.prompt);
      }),

    /**
     * Submit a user response to a planning session.
     */
    respond: publicProcedure
      .input(
        z.object({
          sessionId: z.string(),
          promptId: z.string(),
          response: z.string(),
        })
      )
      .mutation(async ({ input, ctx }) => {
        if (!ctx.planningService) {
          throw new TRPCError({
            code: "INTERNAL_SERVER_ERROR",
            message: "PlanningService is not configured",
          });
        }

        await ctx.planningService.submitResponse(
          input.sessionId,
          input.promptId,
          input.response
        );
        return { success: true };
      }),

    /**
     * Resume a paused planning session.
     */
    resume: publicProcedure
      .input(z.object({ id: z.string() }))
      .mutation(async ({ input, ctx }) => {
        if (!ctx.planningService) {
          throw new TRPCError({
            code: "INTERNAL_SERVER_ERROR",
            message: "PlanningService is not configured",
          });
        }

        await ctx.planningService.resumeSession(input.id);
        return { success: true };
      }),

    /**
     * Delete a planning session.
     */
    delete: publicProcedure
      .input(z.object({ id: z.string() }))
      .mutation(async ({ input, ctx }) => {
        if (!ctx.planningService) {
          throw new TRPCError({
            code: "INTERNAL_SERVER_ERROR",
            message: "PlanningService is not configured",
          });
        }

        await ctx.planningService.deleteSession(input.id);
        return { success: true };
      }),

    /**
     * Get artifact content for a planning session.
     */
    getArtifact: publicProcedure
      .input(z.object({ sessionId: z.string(), filename: z.string() }))
      .query(async ({ input, ctx }) => {
        if (!ctx.planningService) {
          throw new TRPCError({
            code: "INTERNAL_SERVER_ERROR",
            message: "PlanningService is not configured",
          });
        }

        try {
          return await ctx.planningService.getArtifact(
            input.sessionId,
            input.filename
          );
        } catch (error) {
          throw new TRPCError({
            code: "NOT_FOUND",
            message:
              error instanceof Error ? error.message : "Artifact not found",
          });
        }
      }),
  }),
});

export type AppRouter = typeof appRouter;
