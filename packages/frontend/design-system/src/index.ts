/* Shared materials (edge-light tokens + utilities) — load with package entry */
import "./materials.css";

export { Button } from "./button";
export type { ButtonProps, ButtonVariant, ButtonSize } from "./button";
export { Badge } from "./badge";
export { FloatingCountBadge } from "./floating-count-badge";
export type {
  FloatingCountBadgeProps,
  FloatingCountBadgeTone,
} from "./floating-count-badge";
export { Card, Surface } from "./card";
export { Separator } from "./separator";
export { StatusDot, StatusChip } from "./status";
export type { StatusChipAppearance, StatusTone } from "./status";
export {
  DitherIconTile,
  DITHER_BRAND_ACCENTS,
  resolveDitherIconAccent,
} from "./dither-icon";
export type {
  DitherIconSize,
  DitherIconTileProps,
  DitherIconTone,
} from "./dither-icon";
export { TemplateCard, TemplateCardGrid } from "./template-card";
export type {
  TemplateCardGridItem,
  TemplateCardGridProps,
  TemplateCardProps,
} from "./template-card";
export { PageLoading, PageEmpty, PageError, FixtureBanner, PageOffline, PageForbidden } from "./feedback";
export { ToastRegion, pushToast } from "./toast";
export type { ToastItem } from "./toast";
export { PageHeader, PageContent, PageToolbar } from "./page";
export type { PageToolbarRootProps, PageToolbarSlotProps } from "./page";
export { ToolbarIconButton } from "./toolbar-icon-button";
export type { ToolbarIconButtonProps } from "./toolbar-icon-button";
export { SearchBar } from "./search-bar";
export type { SearchBarProps } from "./search-bar";
export { Tabs, TabPanel } from "./tabs";
export type { TabItem } from "./tabs";
export { Drawer } from "./drawer";
export { Popover } from "./popover";
export { Tooltip } from "./tooltip";
export type {
  TooltipContentProps,
  TooltipGroupProps,
  TooltipPlacement,
  TooltipRootProps,
  TooltipTriggerProps,
} from "./tooltip";
export { Kbd } from "./kbd";
export type { KbdProps } from "./kbd";
export { CheckboxRow } from "./checkbox-row";
export type { CheckboxRowProps } from "./checkbox-row";
export { FilterPopover, FilterCategoryPopover } from "./filter-popover";
export type {
  FilterPopoverProps,
  FilterPopoverGroup,
  FilterPopoverOption,
  FilterPopoverAction,
  FilterPopoverSearch,
  FilterCategoryOption,
  FilterCategoryPopoverProps,
} from "./filter-popover";
export { DataTable, DataTableSelectionToolbarHost } from "./table";
export type {
  DataTableColumnHeaderProps,
  DataTablePrimaryCellProps,
  DataTableRootProps,
  DataTableSelectionMode,
  DataTableSelectionState,
  DataTableSelectionToolbarProps,
  DataTableSelectionToolbarState,
  DataTableSortDirection,
  DataTableSortState,
  DataTableSortValue,
} from "./table";
export { DetailPanel } from "./detail-panel";
export type {
  DetailPanelActionProps,
  DetailPanelActionTone,
  DetailPanelBodyProps,
  DetailPanelMetricProps,
  DetailPanelMode,
  DetailPanelPresence,
  DetailPanelRootProps,
  DetailPanelSectionProps,
  DetailPanelSectionSpan,
} from "./detail-panel";
export { MarkdownEditor } from "./markdown-editor";
export type { MarkdownEditorProps } from "./markdown-editor";
export { ResizableSidebar, ResizableSplit } from "../panels/resizable";
export type { ResizableSidebarProps } from "../panels/resizable";
export { A11yDialog, Dialog } from "./dialog";
export type { A11yDialogProps, DialogSize } from "./dialog";
export { Input, Textarea } from "./input";
export type { InputProps, TextareaProps, ControlFieldSize } from "./input";
export { Select } from "./select";
export type { SelectProps, SelectOption } from "./select";
export { Field } from "./field";
export { MenuPanel, MenuItem } from "./menu";
export type { MenuItemProps } from "./menu";
export {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
} from "./dropdown-menu";
export type { DropdownMenuProps } from "./dropdown-menu";
export { ContextMenu } from "./context-menu";
export type { ContextMenuRootProps } from "./context-menu";
export { FilterChip, FilterChipGroup } from "./filter-chip";
export type { FilterChipItem } from "./filter-chip";
export * from "./hugeicons";
