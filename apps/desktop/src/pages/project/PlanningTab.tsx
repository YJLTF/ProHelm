import { Card } from "../../components/ui/Card";
import { TaskList } from "../../components/tasks/TaskList";
import type { Task } from "../../types";

interface Props {
  projectId: string;
  tasks: Task[];
  tasksLoading: boolean;
}

export function PlanningTab({ projectId, tasks, tasksLoading }: Props) {
  return (
    <Card
      title="任务与 WBS"
      subtitle="对应 PMBOK 8「规划」焦点领域 · WBS 编码自动维护（1 / 1.1 / 1.2.3）· 甘特图与关键路径 S2 上线"
    >
      <TaskList projectId={projectId} tasks={tasks} loading={tasksLoading} />
    </Card>
  );
}
