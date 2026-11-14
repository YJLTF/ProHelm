/// frappe-gantt 1.x 类型声明（包未自带 d.ts）。
declare module "frappe-gantt" {
  export interface GanttTask {
    id: string;
    name: string;
    start: string;
    end: string;
    progress?: number;
    /** 逗号分隔的前置任务 id */
    dependencies?: string;
    custom_class?: string;
    [key: string]: unknown;
  }

  export interface GanttOptions {
    view_mode?: "Quarter Day" | "Half Day" | "Day" | "Week" | "Month" | "Year";
    date_format?: string;
    language?: string;
    readonly?: boolean;
    on_click?: (task: GanttTask) => void;
    on_date_change?: (task: GanttTask, start: string, end: string) => void;
    on_progress_change?: (task: GanttTask, progress: number) => void;
    on_view_change?: (mode: string) => void;
    [key: string]: unknown;
  }

  export default class Gantt {
    constructor(wrapper: HTMLElement | string, tasks: GanttTask[], options?: GanttOptions);
    change_view_mode(mode: string): void;
  }
}
