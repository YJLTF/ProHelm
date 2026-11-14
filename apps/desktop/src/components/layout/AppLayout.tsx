import { NavLink, Outlet } from "react-router";
import type { LucideIcon } from "lucide-react";
import {
  Compass,
  FolderKanban,
  LayoutTemplate,
  PanelLeftClose,
  PanelLeftOpen,
  Settings,
  ShipWheel,
} from "lucide-react";

import { cn } from "../../lib/format";
import { useToastStore } from "../../store/toast";
import { useUiStore } from "../../store/ui";

interface NavItem {
  to: string;
  label: string;
  icon: LucideIcon;
  end?: boolean;
}

const navItems: NavItem[] = [
  { to: "/", label: "驾驶舱", icon: Compass, end: true },
  { to: "/projects", label: "项目", icon: FolderKanban },
  { to: "/templates", label: "模板库", icon: LayoutTemplate },
  { to: "/settings", label: "设置", icon: Settings },
];

export function AppLayout() {
  const collapsed = useUiStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);
  const toastMessage = useToastStore((s) => s.message);
  const toastTone = useToastStore((s) => s.tone);

  return (
    <div className="flex h-full">
      <aside
        className={cn(
          "flex shrink-0 flex-col border-r border-slate-200 bg-white transition-all duration-200",
          collapsed ? "w-16" : "w-52",
        )}
      >
        <div className="flex items-center gap-2.5 px-4 py-4">
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-blue-600 text-white">
            <ShipWheel size={18} />
          </div>
          {!collapsed && (
            <div className="min-w-0">
              <div className="text-sm font-bold text-slate-800">项舵 ProHelm</div>
              <div className="truncate text-[10px] text-slate-400">PMP 驱动的项目驾驶舱</div>
            </div>
          )}
        </div>

        <nav className="flex-1 space-y-1 px-2">
          {navItems.map(({ to, label, icon: Icon, end }) => (
            <NavLink
              key={to}
              to={to}
              end={end}
              title={label}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm font-medium",
                  isActive
                    ? "bg-blue-50 text-blue-700"
                    : "text-slate-600 hover:bg-slate-100 hover:text-slate-800",
                )
              }
            >
              <Icon size={17} className="shrink-0" />
              {!collapsed && <span>{label}</span>}
            </NavLink>
          ))}
        </nav>

        <div className="border-t border-slate-100 p-2">
          <button
            onClick={toggleSidebar}
            title={collapsed ? "展开侧栏" : "收起侧栏"}
            className="flex w-full items-center justify-center rounded-lg px-3 py-2 text-slate-400 hover:bg-slate-100 hover:text-slate-600"
          >
            {collapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />}
          </button>
        </div>
      </aside>

      <main className="min-w-0 flex-1 overflow-y-auto bg-slate-50">
        <Outlet />
      </main>

      {toastMessage && (
        <div
          className={cn(
            "fixed bottom-5 left-1/2 z-50 -translate-x-1/2 rounded-lg px-4 py-2 text-sm text-white shadow-lg",
            toastTone === "error" ? "bg-red-600" : "bg-slate-800",
          )}
        >
          {toastMessage}
        </div>
      )}
    </div>
  );
}
