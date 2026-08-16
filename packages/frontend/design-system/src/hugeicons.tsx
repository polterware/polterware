import { For, splitProps, type Component, type JSX } from "solid-js";
import { Dynamic } from "solid-js/web";
import {
  Activity as ActivityDefinition,
  Activity03Icon as Activity03IconDefinition,
  AddTeamIcon as AddTeamDefinition,
  Archive as ArchiveDefinition,
  ArrowDown as ArrowDownDefinition,
  ArrowUp as ArrowUpDefinition,
  Bell as BellDefinition,
  BlocksIcon as BlocksDefinition,
  Book03Icon as Book03IconDefinition,
  BookOpen as BookOpenDefinition,
  BookOpen02Icon as BookOpen02IconDefinition,
  Bookshelf01Icon as Bookshelf01IconDefinition,
  Bot as BotDefinition,
  Box as BoxDefinition,
  Boxes as BoxesDefinition,
  Building2 as Building2Definition,
  CalendarClock as CalendarClockDefinition,
  Check as CheckDefinition,
  ChevronDown as ChevronDownDefinition,
  ChevronLeft as ChevronLeftDefinition,
  ChevronRight as ChevronRightDefinition,
  Clock as ClockDefinition,
  Cloud as CloudDefinition,
  CloudCog as CloudCogDefinition,
  Cog as CogDefinition,
  CollapseIcon as CollapseIconDefinition,
  ConnectIcon as ConnectIconDefinition,
  Copy02Icon as CopyDefinition,
  Cpu as CpuDefinition,
  CreditCard as CreditCardDefinition,
  Database as DatabaseDefinition,
  Delete02Icon as DeleteDefinition,
  Download as DownloadDefinition,
  ExternalLink as ExternalLinkDefinition,
  Eye as EyeDefinition,
  ExpandIcon as ExpandIconDefinition,
  File as FileDefinition,
  FileDiffIcon as FileDiffDefinition,
  FileText as FileTextDefinition,
  FileUp as FileUpDefinition,
  FloppyDiskIcon as FloppyDiskDefinition,
  FingerAccessIcon as FingerAccessIconDefinition,
  Folder as FolderDefinition,
  FolderKanbanIcon as FolderKanbanDefinition,
  GaugeIcon as GaugeDefinition,
  GitBranch as GitBranchDefinition,
  GitCompare as GitCompareDefinition,
  GitMerge as GitMergeDefinition,
  GitPullRequest as GitPullRequestDefinition,
  Globe as GlobeDefinition,
  Globe02Icon as Globe2Definition,
  Hammer as HammerDefinition,
  Hand as HandDefinition,
  HardDrive as HardDriveDefinition,
  History as HistoryDefinition,
  House as HouseDefinition,
  Inbox as InboxDefinition,
  KeyRound as KeyRoundDefinition,
  Laptop as LaptopDefinition,
  Layers as LayersDefinition,
  LayoutDashboard as LayoutDashboardDefinition,
  LibraryBig as LibraryBigDefinition,
  Link as LinkDefinition,
  Link2 as Link2Definition,
  ListChecks as ListChecksDefinition,
  ListFilter as ListFilterDefinition,
  ListOrdered as ListOrderedDefinition,
  MapPin as MapPinDefinition,
  Maximize2 as Maximize2Definition,
  MemoryStick as MemoryStickDefinition,
  Monitor as MonitorDefinition,
  MousePointer2 as MousePointer2Definition,
  Network as NetworkDefinition,
  Package as PackageDefinition,
  PauseIcon as PauseDefinition,
  Pencil as PencilDefinition,
  Play as PlayDefinition,
  Plug as PlugDefinition,
  Plus as PlusDefinition,
  Redo2 as Redo2Definition,
  RefreshCw as RefreshCwDefinition,
  Route as RouteDefinition,
  Scale as ScaleDefinition,
  SchemeIcon as SchemeIconDefinition,
  ScrollText as ScrollTextDefinition,
  Search as SearchDefinition,
  Server as ServerDefinition,
  Settings as SettingsDefinition,
  Shield as ShieldDefinition,
  ShieldCheck as ShieldCheckDefinition,
  Smartphone as SmartphoneDefinition,
  Square as SquareDefinition,
  SquareArrowRight02Icon as SquareChevronRightDefinition,
  ComputerTerminal01Icon as SquareTerminalDefinition,
  StickyNote as StickyNoteDefinition,
  StopIcon as StopDefinition,
  Terminal as TerminalDefinition,
  ThumbsDownIcon as ThumbsDownIconDefinition,
  ThumbsUpIcon as ThumbsUpIconDefinition,
  Undo2 as Undo2Definition,
  Warehouse as WarehouseDefinition,
  Webhook as WebhookDefinition,
  Workflow as WorkflowDefinition,
  X as XDefinition,
  Zap as ZapDefinition,
  MoreHorizontalIcon as MoreHorizontalDefinition,
  TextBoldIcon as TextBoldIconDefinition,
  TextItalicIcon as TextItalicIconDefinition,
  Heading02Icon as Heading02IconDefinition,
  LeftToRightListBulletIcon as LeftToRightListBulletIconDefinition,
  LeftToRightListNumberIcon as LeftToRightListNumberIconDefinition,
  SourceCodeIcon as SourceCodeIconDefinition,
  QuoteDownIcon as QuoteDownIconDefinition,
  MinusSignIcon as MinusSignIconDefinition,
} from "@hugeicons/core-free-icons";

type HugeIconNodeAttributes = Readonly<Record<string, string | number>>;

export type HugeIconDefinition = readonly (
  readonly [tag: string, attributes: HugeIconNodeAttributes]
)[];

export const POLTER_ICON_STROKE_WIDTH = 1.5;

export interface HugeiconsIconProps
  extends Omit<JSX.SvgSVGAttributes<SVGSVGElement>, "children"> {
  icon: HugeIconDefinition;
  size?: string | number;
  strokeWidth?: number;
}

export type HugeiconComponent = Component<
  Omit<HugeiconsIconProps, "icon">
>;

interface DecorativeSvgAttributes extends JSX.SvgSVGAttributes<SVGSVGElement> {
  focusable: "false";
}

const DECORATIVE_SVG_ATTRIBUTES: DecorativeSvgAttributes = {
  focusable: "false",
};

interface NormalizedNodeAttributes {
  [key: string]: string | number;
  "stroke-width": number;
}

function normalizeNodeAttributes(
  attributes: HugeIconNodeAttributes,
  strokeWidth: number,
): NormalizedNodeAttributes {
  const {
    key: _key,
    strokeLinecap,
    strokeLinejoin,
    strokeWidth: _definitionStrokeWidth,
    ...rest
  } = attributes;

  const result: NormalizedNodeAttributes = {
    ...rest,
    "stroke-width": strokeWidth,
  };
  if (strokeLinecap !== undefined) result["stroke-linecap"] = strokeLinecap;
  if (strokeLinejoin !== undefined) result["stroke-linejoin"] = strokeLinejoin;
  return result;
}

export function HugeiconsIcon(props: HugeiconsIconProps): JSX.Element {
  const [local, svgProps] = splitProps(props, [
    "icon",
    "size",
    "strokeWidth",
    "class",
    "color",
    "aria-hidden",
  ]);
  const size = () => local.size ?? 24;
  const strokeWidth = () =>
    local.strokeWidth ?? POLTER_ICON_STROKE_WIDTH;

  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size()}
      height={size()}
      viewBox="0 0 24 24"
      fill="none"
      color={local.color ?? "currentColor"}
      class={local.class}
      aria-hidden={local["aria-hidden"] ?? "true"}
      {...svgProps}
      {...DECORATIVE_SVG_ATTRIBUTES}
    >
      <For each={local.icon}>
        {([tag, attributes]) => (
          <Dynamic
            component={tag}
            {...normalizeNodeAttributes(attributes, strokeWidth())}
          />
        )}
      </For>
    </svg>
  );
}

function createHugeicon(
  definition: HugeIconDefinition,
): HugeiconComponent {
  return (props) => <HugeiconsIcon {...props} icon={definition} />;
}

const iconDefinitions = {
  Activity: ActivityDefinition,
  Activity03Icon: Activity03IconDefinition,
  AddTeam: AddTeamDefinition,
  Archive: ArchiveDefinition,
  ArrowDown: ArrowDownDefinition,
  ArrowUp: ArrowUpDefinition,
  Bell: BellDefinition,
  Blocks: BlocksDefinition,
  Book03Icon: Book03IconDefinition,
  BookOpen: BookOpenDefinition,
  BookOpen02Icon: BookOpen02IconDefinition,
  Bookshelf01Icon: Bookshelf01IconDefinition,
  Bot: BotDefinition,
  Box: BoxDefinition,
  Boxes: BoxesDefinition,
  Building2: Building2Definition,
  CalendarClock: CalendarClockDefinition,
  XIcon: XDefinition,
  Check: CheckDefinition,
  ChevronDown: ChevronDownDefinition,
  ChevronLeft: ChevronLeftDefinition,
  ChevronRight: ChevronRightDefinition,
  Clock: ClockDefinition,
  Cloud: CloudDefinition,
  CloudCog: CloudCogDefinition,
  Cog: CogDefinition,
  CollapseIcon: CollapseIconDefinition,
  ConnectIcon: ConnectIconDefinition,
  Copy: CopyDefinition,
  Cpu: CpuDefinition,
  CreditCard: CreditCardDefinition,
  Database: DatabaseDefinition,
  Delete: DeleteDefinition,
  Download: DownloadDefinition,
  ExternalLink: ExternalLinkDefinition,
  Eye: EyeDefinition,
  ExpandIcon: ExpandIconDefinition,
  File: FileDefinition,
  FileDiff: FileDiffDefinition,
  FileText: FileTextDefinition,
  FileUp: FileUpDefinition,
  FloppyDisk: FloppyDiskDefinition,
  FingerAccessIcon: FingerAccessIconDefinition,
  Folder: FolderDefinition,
  FolderKanban: FolderKanbanDefinition,
  Gauge: GaugeDefinition,
  GitBranch: GitBranchDefinition,
  GitCompare: GitCompareDefinition,
  GitMerge: GitMergeDefinition,
  GitPullRequest: GitPullRequestDefinition,
  Globe: GlobeDefinition,
  Globe2: Globe2Definition,
  Hammer: HammerDefinition,
  Hand: HandDefinition,
  HardDrive: HardDriveDefinition,
  History: HistoryDefinition,
  House: HouseDefinition,
  Inbox: InboxDefinition,
  KeyRound: KeyRoundDefinition,
  Laptop: LaptopDefinition,
  Layers: LayersDefinition,
  LayoutDashboard: LayoutDashboardDefinition,
  LibraryBig: LibraryBigDefinition,
  Link: LinkDefinition,
  Link2: Link2Definition,
  ListChecks: ListChecksDefinition,
  ListFilter: ListFilterDefinition,
  ListOrdered: ListOrderedDefinition,
  MapPin: MapPinDefinition,
  Maximize2: Maximize2Definition,
  MemoryStick: MemoryStickDefinition,
  Monitor: MonitorDefinition,
  MousePointer2: MousePointer2Definition,
  Network: NetworkDefinition,
  Package: PackageDefinition,
  Pause: PauseDefinition,
  Pencil: PencilDefinition,
  Play: PlayDefinition,
  Plug: PlugDefinition,
  Plus: PlusDefinition,
  Redo2: Redo2Definition,
  RefreshCw: RefreshCwDefinition,
  Route: RouteDefinition,
  Scale: ScaleDefinition,
  SchemeIcon: SchemeIconDefinition,
  ScrollText: ScrollTextDefinition,
  Search: SearchDefinition,
  Server: ServerDefinition,
  Settings: SettingsDefinition,
  Shield: ShieldDefinition,
  ShieldCheck: ShieldCheckDefinition,
  Smartphone: SmartphoneDefinition,
  Square: SquareDefinition,
  SquareChevronRight: SquareChevronRightDefinition,
  SquareTerminal: SquareTerminalDefinition,
  StickyNote: StickyNoteDefinition,
  Stop: StopDefinition,
  Terminal: TerminalDefinition,
  ThumbsDownIcon: ThumbsDownIconDefinition,
  ThumbsUpIcon: ThumbsUpIconDefinition,
  Undo2: Undo2Definition,
  Warehouse: WarehouseDefinition,
  Webhook: WebhookDefinition,
  Workflow: WorkflowDefinition,
  X: XDefinition,
  Zap: ZapDefinition,
  MoreHorizontal: MoreHorizontalDefinition,
  TextBoldIcon: TextBoldIconDefinition,
  TextItalicIcon: TextItalicIconDefinition,
  Heading02Icon: Heading02IconDefinition,
  LeftToRightListBulletIcon: LeftToRightListBulletIconDefinition,
  LeftToRightListNumberIcon: LeftToRightListNumberIconDefinition,
  SourceCodeIcon: SourceCodeIconDefinition,
  QuoteDownIcon: QuoteDownIconDefinition,
  MinusSignIcon: MinusSignIconDefinition,
} satisfies Record<string, HugeIconDefinition>;

type PolterIconName = keyof typeof iconDefinitions;

// SAFETY: We map the exact keys of iconDefinitions to their corresponding HugeiconComponent.
const iconComponents = Object.fromEntries(
  Object.entries(iconDefinitions).map(([name, definition]) => [
    name,
    createHugeicon(definition),
  ]),
) as Record<PolterIconName, HugeiconComponent>;

export const {
  Activity,
  Activity03Icon,
  AddTeam,
  Archive,
  ArrowDown,
  ArrowUp,
  Bell,
  Blocks,
  Book03Icon,
  BookOpen,
  BookOpen02Icon,
  Bookshelf01Icon,
  Bot,
  Box,
  Boxes,
  Building2,
  CalendarClock,
  XIcon,
  Check,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Clock,
  Cloud,
  CloudCog,
  Cog,
  CollapseIcon,
  ConnectIcon,
  Copy,
  Cpu,
  CreditCard,
  Database,
  Delete,
  Download,
  ExternalLink,
  Eye,
  ExpandIcon,
  File,
  FileDiff,
  FileText,
  FileUp,
  FloppyDisk,
  FingerAccessIcon,
  Folder,
  FolderKanban,
  Gauge,
  GitBranch,
  GitCompare,
  GitMerge,
  GitPullRequest,
  Globe,
  Globe2,
  Hammer,
  Hand,
  HardDrive,
  History,
  House,
  Inbox,
  KeyRound,
  Laptop,
  Layers,
  LayoutDashboard,
  LibraryBig,
  Link,
  Link2,
  ListChecks,
  ListFilter,
  ListOrdered,
  MapPin,
  Maximize2,
  MemoryStick,
  Monitor,
  MousePointer2,
  Network,
  Package,
  Pause,
  Pencil,
  Play,
  Plug,
  Plus,
  Redo2,
  RefreshCw,
  Route,
  Scale,
  SchemeIcon,
  ScrollText,
  Search,
  Server,
  Settings,
  Shield,
  ShieldCheck,
  Smartphone,
  Square,
  SquareChevronRight,
  SquareTerminal,
  StickyNote,
  Stop,
  Terminal,
  ThumbsDownIcon,
  ThumbsUpIcon,
  Undo2,
  Warehouse,
  Webhook,
  Workflow,
  X,
  Zap,
  MoreHorizontal,
  TextBoldIcon,
  TextItalicIcon,
  Heading02Icon,
  LeftToRightListBulletIcon,
  LeftToRightListNumberIcon,
  SourceCodeIcon,
  QuoteDownIcon,
  MinusSignIcon,
} = iconComponents;

export const PlusIcon = Plus;
export const ExternalLinkIcon = ExternalLink;
