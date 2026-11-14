import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api } from "../lib/ipc";
import { toastError } from "../store/toast";
import type {
  CreateProjectInput,
  CreateTaskInput,
  UpdateProjectInput,
  UpdateTaskInput,
} from "../types";

// ---------- 查询 ----------

export function useProjects() {
  return useQuery({ queryKey: ["projects"], queryFn: api.listProjects });
}

export function useProject(id: string) {
  return useQuery({
    queryKey: ["projects", id],
    queryFn: async () => {
      const list = await api.listProjects();
      return list.find((p) => p.id === id) ?? null;
    },
    enabled: id.length > 0,
  });
}

export function useDashboardStats() {
  return useQuery({ queryKey: ["dashboard-stats"], queryFn: api.dashboardStats });
}

export function useTasks(projectId: string) {
  return useQuery({
    queryKey: ["tasks", projectId],
    queryFn: () => api.listTasks(projectId),
    enabled: projectId.length > 0,
  });
}

// ---------- 变更 ----------

export function useCreateProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateProjectInput) => api.createProject(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["projects"] });
      void qc.invalidateQueries({ queryKey: ["dashboard-stats"] });
    },
    onError: (e: Error) => toastError(e.message),
  });
}

export function useUpdateProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateProjectInput) => api.updateProject(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["projects"] });
      void qc.invalidateQueries({ queryKey: ["dashboard-stats"] });
    },
    onError: (e: Error) => toastError(e.message),
  });
}

export function useDeleteProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteProject(id),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["projects"] });
      void qc.invalidateQueries({ queryKey: ["dashboard-stats"] });
    },
    onError: (e: Error) => toastError(e.message),
  });
}

function invalidateTasks(qc: ReturnType<typeof useQueryClient>, projectId: string) {
  void qc.invalidateQueries({ queryKey: ["tasks", projectId] });
  void qc.invalidateQueries({ queryKey: ["dashboard-stats"] });
}

export function useCreateTask(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateTaskInput) => api.createTask(input),
    onSuccess: () => invalidateTasks(qc, projectId),
    onError: (e: Error) => toastError(e.message),
  });
}

export function useUpdateTask(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateTaskInput) => api.updateTask(input),
    onSuccess: () => invalidateTasks(qc, projectId),
    onError: (e: Error) => toastError(e.message),
  });
}

export function useDeleteTask(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteTask(id),
    onSuccess: () => invalidateTasks(qc, projectId),
    onError: (e: Error) => toastError(e.message),
  });
}
