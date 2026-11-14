export function cn(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

export function formatDate(iso: string | null | undefined): string {
  if (!iso) return "—";
  return iso.slice(0, 10);
}

export function formatBudget(v: number): string {
  if (v <= 0) return "未设预算";
  return `¥${v.toLocaleString("zh-CN")}`;
}

export function statusTone(status: string): "green" | "blue" | "gray" | "amber" {
  switch (status) {
    case "active":
      return "green";
    case "planning":
      return "blue";
    case "paused":
      return "amber";
    default:
      return "gray";
  }
}

export function priorityTone(p: string): "gray" | "blue" | "amber" | "red" {
  switch (p) {
    case "urgent":
      return "red";
    case "high":
      return "amber";
    case "medium":
      return "blue";
    default:
      return "gray";
  }
}
