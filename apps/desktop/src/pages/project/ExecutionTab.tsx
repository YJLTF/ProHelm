import {
  DndContext,
  PointerSensor,
  useDraggable,
  useDroppable,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import type { DragEndEvent } from "@dnd-kit/core";
import { ArrowLeft, ArrowRight, GripVertical } from "lucide-react";

import { Badge } from "../../components/ui/Badge";
import { Card } from "../../components/ui/Card";
import { useUpdateTask } from "../../hooks/useProjects";
import { cn, formatDate, priorityTone } from "../../lib/format";
import { TASK_PRIORITY_LABEL, TASK_STATUS_LABEL } from "../../types";
import type { Task, TaskStatus } from "../../types";

interface Props {
  projectId: string;
  tasks: Task[];
}

const FLOW: TaskStatus[] = ["todo", "in_progress", "done"];

const COLUMNS: Array<{ key: TaskStatus; label: string; accent: string }> = [
  { key: "todo", label: TASK_STATUS_LABEL.todo, accent: "bg-slate-400" },
  { key: "in_progress", label: TASK_STATUS_LABEL.in_progress, accent: "bg-blue-500" },
  { key: "done", label: TASK_STATUS_LABEL.done, accent: "bg-emerald-500" },
];

/** 执行页签 = PMBOK 8「执行」焦点领域：dnd-kit 拖拽流转 */
export function ExecutionTab({ projectId, tasks }: Props) {
  const update = useUpdateTask(projectId);
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }));

  const move = (task: Task, dir: -1 | 1) => {
    const idx = FLOW.indexOf(task.status);
    const next = FLOW[idx + dir];
    if (next) update.mutate({ id: task.id, status: next });
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over) return;
    const taskId = String(active.id);
    const target = String(over.id) as TaskStatus;
    if (!COLUMNS.some((c) => c.key === target)) return;
    const task = tasks.find((t) => t.id === taskId);
    if (task && task.status !== target) {
      update.mutate({ id: taskId, status: target });
    }
  };

  return (
    <Card
      title="任务看板"
      subtitle="对应 PMBOK 8「执行」焦点领域 · 拖拽卡片跨列流转（也可用箭头按钮）"
    >
      <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
        <div className="grid grid-cols-3 gap-3">
          {COLUMNS.map((col) => (
            <Column key={col.key} column={col} tasks={tasks} onMove={move} />
          ))}
        </div>
      </DndContext>
      {tasks.length === 0 && (
        <p className="mt-3 text-center text-xs text-slate-400">
          先在「规划」页签创建任务，再回到这里流转。
        </p>
      )}
    </Card>
  );
}

function Column({
  column,
  tasks,
  onMove,
}: {
  column: { key: TaskStatus; label: string; accent: string };
  tasks: Task[];
  onMove: (task: Task, dir: -1 | 1) => void;
}) {
  const items = tasks.filter((t) => t.status === column.key);
  const { setNodeRef, isOver } = useDroppable({ id: column.key });

  return (
    <div
      ref={setNodeRef}
      className={cn(
        "rounded-xl p-2.5 transition-colors",
        isOver ? "bg-blue-50 ring-2 ring-blue-200" : "bg-slate-50",
      )}
    >
      <div className="mb-2 flex items-center gap-2 px-1">
        <span className={cn("h-2 w-2 rounded-full", column.accent)} />
        <span className="text-xs font-semibold text-slate-600">{column.label}</span>
        <span className="text-xs text-slate-400">{items.length}</span>
      </div>
      <div className="min-h-16 space-y-2">
        {items.length === 0 ? (
          <div className="rounded-lg border border-dashed border-slate-200 py-5 text-center text-[11px] text-slate-300">
            拖拽卡片到这里
          </div>
        ) : (
          items.map((t) => (
            <TaskCard key={t.id} task={t} onMove={onMove} />
          ))
        )}
      </div>
    </div>
  );
}

function TaskCard({ task, onMove }: { task: Task; onMove: (task: Task, dir: -1 | 1) => void }) {
  const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({
    id: task.id,
  });
  const idx = FLOW.indexOf(task.status);

  return (
    <div
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      style={{
        transform: transform ? `translate3d(${transform.x}px, ${transform.y}px, 0)` : undefined,
        opacity: isDragging ? 0.45 : 1,
      }}
      className={cn(
        "cursor-grab rounded-lg border bg-white p-2.5 shadow-sm transition-colors active:cursor-grabbing",
        isDragging ? "border-blue-300" : "border-slate-200 hover:border-slate-300",
      )}
    >
      <div className="flex items-center gap-1.5">
        <GripVertical size={12} className="shrink-0 text-slate-300" />
        <span className="font-mono text-[10px] text-slate-400">{task.wbsCode}</span>
        <Badge tone={priorityTone(task.priority)}>{TASK_PRIORITY_LABEL[task.priority]}</Badge>
      </div>
      <div className="mt-1 line-clamp-2 text-sm text-slate-700">{task.title}</div>
      <div className="mt-1.5 flex items-center justify-between text-[11px] text-slate-400">
        <span>{task.assignee ?? "未分配"}</span>
        <span>{formatDate(task.endDate)}</span>
      </div>
      <div className="mt-2 flex justify-end gap-1">
        {idx > 0 && (
          <button
            title={`移回${COLUMNS[idx - 1].label}`}
            onClick={() => onMove(task, -1)}
            className="rounded p-1 text-slate-400 hover:bg-slate-100 hover:text-slate-600"
          >
            <ArrowLeft size={13} />
          </button>
        )}
        {idx < FLOW.length - 1 && (
          <button
            title={`推进到${COLUMNS[idx + 1].label}`}
            onClick={() => onMove(task, 1)}
            className="rounded p-1 text-slate-400 hover:bg-slate-100 hover:text-blue-600"
          >
            <ArrowRight size={13} />
          </button>
        )}
      </div>
    </div>
  );
}
