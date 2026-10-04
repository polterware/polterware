import { splitProps, type JSX } from "solid-js";
import { Button, type ButtonProps } from "./button";
import "./calendar-grid.css";
export type CalendarGridProps = { view: "day" | "week" | "month"; label: string; children: JSX.Element; class?: string; hourHeight?: number; allDayHeight?: number; gutterWidth?: number };
function Root(props: CalendarGridProps) {
  return <section aria-label={props.label} data-view={props.view} class={`pw-calendar ${props.class ?? ""}`} style={{ "--calendar-hour-height": `${props.hourHeight ?? 56}px`, "--calendar-all-day-height": `${props.allDayHeight ?? 36}px`, "--calendar-gutter-width": `${props.gutterWidth ?? 48}px` }}>{props.children}</section>;
}
function Days(props: { children: JSX.Element; class?: string }) { return <div class={`pw-calendar__days ${props.class ?? ""}`}>{props.children}</div>; }
function Day(props: { label: string; today?: boolean; children: JSX.Element }) { return <section aria-label={props.label} data-today={props.today ? "" : undefined} class="pw-calendar__day">{props.children}</section>; }
function Header(props: { children: JSX.Element }) { return <header class="pw-calendar__header">{props.children}</header>; }
function AllDay(props: { label?: string; children: JSX.Element }) { return <div aria-label={props.label ?? "All day"} class="pw-calendar__all-day">{props.children}</div>; }
function HourRow(props: { label: string; children?: JSX.Element }) { return <div class="pw-calendar__hour"><span class="pw-calendar__hour-label">{props.label}</span><div class="pw-calendar__hour-content">{props.children}</div></div>; }
function Occurrence(props: ButtonProps & { overdue?: boolean; completed?: boolean }) {
  const [local, rest] = splitProps(props, ["overdue", "completed", "class"]);
  return <Button {...rest} variant="ghost" class={`pw-calendar__occurrence ${local.class ?? ""}`} data-overdue={local.overdue ? "" : undefined} data-completed={local.completed ? "" : undefined} />;
}
function Overflow(props: { count: number; onClick?: () => void; label?: string }) { return <Button variant="ghost" size="sm" onClick={props.onClick} aria-label={props.label ?? `${props.count} more occurrences`}>+{props.count}</Button>; }
export const CalendarGrid = { Root, Days, Day, Header, AllDay, HourRow, Occurrence, Overflow };
