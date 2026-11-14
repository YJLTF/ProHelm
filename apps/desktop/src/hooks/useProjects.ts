import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api } from "../lib/ipc";
import { toastError } from "../store/toast";
import type {
  CharterContent,
  CreateDependencyInput,
  CreateProjectInput,
  CreateRiskInput,
  CreateStakeholderInput,
  CreateTaskInput,
  MoveTaskInput,
  ReorderTaskInput,
  UpdateProjectInput,
  UpdateRiskInput,
  UpdateStakeholderInput,
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

// ---------- S2：排程 / 依赖 / WBS 重排 / 基线 ----------

export function useSchedule(projectId: string) {
  return useQuery({
    queryKey: ["schedule", projectId],
    queryFn: () => api.getSchedule(projectId),
    enabled: projectId.length > 0,
  });
}

function invalidateSchedule(qc: ReturnType<typeof useQueryClient>, projectId: string) {
  void qc.invalidateQueries({ queryKey: ["schedule", projectId] });
}

export function useCreateDependency(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateDependencyInput) => api.createDependency(input),
    onSuccess: () => invalidateSchedule(qc, projectId),
    onError: (e: Error) => toastError(e.message),
  });
}

export function useMoveTask(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: MoveTaskInput) => api.moveTask(input),
    onSuccess: () => {
      invalidateTasks(qc, projectId);
      invalidateSchedule(qc, projectId);
    },
    onError: (e: Error) => toastError(e.message),
  });
}

export function useReorderTask(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: ReorderTaskInput) => api.reorderTask(input),
    onSuccess: () => {
      invalidateTasks(qc, projectId);
      invalidateSchedule(qc, projectId);
    },
    onError: (e: Error) => toastError(e.message),
  });
}

export function useBaselines(projectId: string) {
  return useQuery({
    queryKey: ["baselines", projectId],
    queryFn: () => api.listBaselines(projectId),
    enabled: projectId.length > 0,
  });
}

export function useCreateBaseline(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => api.createBaseline(projectId, name),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["baselines", projectId] });
    },
    onError: (e: Error) => toastError(e.message),
  });
}

// ---------- S3：风险 / 干系人 / 章程 / 裁剪 ----------

export function useRisks(projectId: string) {
  return useQuery({
    queryKey: ["risks", projectId],
    queryFn: () => api.listRisks(projectId),
    enabled: projectId.length > 0,
  });
}

export function useCreateRisk(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateRiskInput) => api.createRisk(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["risks", projectId] });
    },
    onError: (e: Error) => toastError(e.message),
  });
}

export function useUpdateRisk(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateRiskInput) => api.updateRisk(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["risks", projectId] });
    },
    onError: (e: Error) => toastError(e.message),
  });
}

export function useDeleteRisk(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteRisk(id),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["risks", projectId] });
    },
    onError: (e: Error) => toastError(e.message),
  });
}

export function useStakeholders(projectId: string) {
  return useQuery({
    queryKey: ["stakeholders", projectId],
    queryFn: () => api.listStakeholders(projectId),
    enabled: projectId.length > 0,
  });
}

export function useCreateStakeholder(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateStakeholderInput) => api.createStakeholder(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["stakeholders", projectId] });
    },
    onError: (e: Error) => toastError(e.message),
  });
}

export function useUpdateStakeholder(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateStakeholderInput) => api.updateStakeholder(input),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["stakeholders", projectId] });
    },
    onError: (e: Error) => toastError(e.message),
  });
}

export function useDeleteStakeholder(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteStakeholder(id),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["stakeholders", projectId] });
    },
    onError: (e: Error) => toastError(e.message),
  });
}

export function useCharter(projectId: string) {
  return useQuery({
    queryKey: ["charter", projectId],
    queryFn: () => api.getCharter(projectId),
    enabled: projectId.length > 0,
  });
}

export function useSaveCharter(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (content: CharterContent) => api.saveCharter(projectId, content),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["charter", projectId] });
    },
    onError: (e: Error) => toastError(e.message),
  });
}

function invalidateProject(qc: ReturnType<typeof useQueryClient>, projectId: string) {
  void qc.invalidateQueries({ queryKey: ["projects", projectId] });
  void qc.invalidateQueries({ queryKey: ["projects"] });
}

export function useSetTailored(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (keys: string[]) => api.setTailored(projectId, keys),
    onSuccess: () => invalidateProject(qc, projectId),
    onError: (e: Error) => toastError(e.message),
  });
}

export function useApplyRecommendedTailoring(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.applyRecommendedTailoring(projectId),
    onSuccess: () => invalidateProject(qc, projectId),
    onError: (e: Error) => toastError(e.message),
  });
}
