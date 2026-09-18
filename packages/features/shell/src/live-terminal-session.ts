export type LiveTerminalSession = {
  id: string;
  name: string;
  targetId: string;
  nodeName: string;
  shell: string;
  status: "active" | "idle" | "closed";
  startedAt: string;
};
