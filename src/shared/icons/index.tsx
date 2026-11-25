/**
 * App icon set: Hugeicons free tier (stroke), wrapped so views import
 * semantic names from one place — same pattern as the dashboard. Size via
 * className or the `size` prop; colour follows currentColor.
 * Brand marks (Shard, GitHub) stay hand-drawn vector SVGs.
 */
import { HugeiconsIcon } from "@hugeicons/react";
import { ICON_STROKE, cn } from "@proxyshard/shardx-ui-kit";
import {
  ArrowLeft01Icon,
  UserIcon,
  BookmarkAdd01Icon,
  BrowserIcon,
  Route01Icon,
  ShoppingCart01Icon,
  FingerPrintIcon,
  Settings01Icon,
  Copy01Icon,
  BookOpen01Icon,
  AppleIcon,
  AndroidIcon,
  WindowsNewIcon,
  TerminalIcon,
  PinIcon,
  PinOffIcon,
  Search01Icon,
  Download04Icon,
  Upload04Icon,
  Sun01Icon,
  Moon02Icon,
  ViewIcon,
  ViewOffSlashIcon,
  Key01Icon,
  Edit02Icon,
  ArrowReloadHorizontalIcon,
  PlusSignIcon,
  Folder01Icon,
  InformationCircleIcon,
  ArrowDown01Icon,
  Delete02Icon,
  Globe02Icon,
  Clock01Icon,
  Building02Icon,
  PlayIcon as HugePlayIcon,
  StopIcon as HugeStopIcon,
  MoreVerticalIcon,
  Link04Icon,
  PauseIcon as HugePauseIcon,
  SparklesIcon,
  StarIcon,
  LockIcon,
  SquareLock02Icon,
  SquareUnlock02Icon,
  ResetPasswordIcon,
  PuzzleIcon,
  Bookmark02Icon,
  Delete03Icon,
  ArchiveRestoreIcon,
  ComputerIcon as HugeComputerIcon,
  CheckmarkCircle02Icon,
  Alert02Icon,
  AlertCircleIcon as HugeAlertCircleIcon,
  KeyboardIcon as HugeKeyboardIcon,
  FloppyDiskIcon,
  Undo02Icon,
  FilterIcon as HugeFilterIcon,
  Cancel01Icon,
  WorkflowSquare03Icon,
  Briefcase01Icon,
  Megaphone01Icon,
  UserGroupIcon,
  CodeIcon,
  GameController01Icon,
  Wallet01Icon,
  Mail01Icon,
  Rocket01Icon,
  Tag01Icon,
  SlidersHorizontalIcon,
  PencilEdit01Icon,
  FolderMinusIcon,
  FolderAddIcon,
  ArrowRight01Icon,
  ColumnsThreeCogIcon,
  ArrowUp02Icon,
  ArrowDown02Icon,
  PencilEdit02Icon,
  ArrowUpRight01Icon,
  SidebarLeftIcon,
  Loading03Icon,
  SidebarRight01Icon,
  IdIcon,
  MagicWand01Icon,
  HelpCircleIcon as HugeHelpCircleIcon,
  MinusSignIcon,
  ArrowUpRight02Icon,
  ArrowDownRight02Icon,
  ArrowRight02Icon,
  Tick02Icon,
  UnfoldMoreIcon as HugeUnfoldMoreIcon,
  CloudSyncIcon,
  CloudOffIcon,
  CloudAlertIcon,
} from "@hugeicons/core-free-icons";
import type { ComponentProps } from "react";

type HugeiconsIconProps = ComponentProps<typeof HugeiconsIcon>;
type IconProps = Omit<HugeiconsIconProps, "icon">;

const hasExplicitSize = (cls?: string) => !!cls && /\b(size-|w-|h-)/.test(cls);

// `name`, when given, binds a stable `data-icon` on the rendered svg once,
// inside the component — not re-typed (and so re-typeable wrong, or onto
// the wrong glyph) at every call site.
//
// Decorative unless told otherwise: the control around it carries the name
// (A11Y-21). A meaningful icon passes `role="img"` and an `aria-label` (or
// `aria-hidden={false}`).
const make = (icon: HugeiconsIconProps["icon"], name?: string) =>
  function AppIcon({ className, ...props }: IconProps) {
    return (
      <HugeiconsIcon
        icon={icon}
        strokeWidth={ICON_STROKE}
        data-icon={name}
        aria-hidden={props.role ? undefined : "true"}
        focusable="false"
        className={cn(!props.size && !hasExplicitSize(className) && "size-[18px]", className)}
        {...props}
      />
    );
  };

/* ── Navigation / section icons ── */
export const NavBrowsersIcon = make(BrowserIcon);
export const NavProxiesIcon = make(Route01Icon);
export const NavShopIcon = make(ShoppingCart01Icon);
export const NavFingerprintsIcon = make(FingerPrintIcon);
export const NavSettingsIcon = make(Settings01Icon);
export const DocsIcon = make(BookOpen01Icon);
export const NavPatchLogIcon = make(SparklesIcon);
export const NavExtensionsIcon = make(PuzzleIcon);
export const GhostModeIcon = make(ViewOffSlashIcon);
export const NavBookmarksIcon = make(Bookmark02Icon);
export const NavTrashIcon = make(Delete03Icon);
export const NavAutomationIcon = make(WorkflowSquare03Icon);
/** The person's own space and a team's, in the sidebar's space switcher. */
export const SpacePersonalIcon = make(UserIcon);
export const SpaceTeamIcon = make(UserGroupIcon);
/** A switch that opens a choice of places (the space row). */
export const UnfoldMoreIcon = make(HugeUnfoldMoreIcon, "unfold-more");
/** Folds the sidebar to its icon rail and back (Mod+\). */
export const SidebarIcon = make(SidebarLeftIcon);

/* ── OS logos ── */
export const AppleOsIcon = make(AppleIcon);
export const WindowsOsIcon = make(WindowsNewIcon);
export const LinuxOsIcon = make(TerminalIcon);
export const AndroidOsIcon = make(AndroidIcon);

/* ── Actions / affordances ── */
export const RouteIcon = make(Route01Icon);
export const SearchIcon = make(Search01Icon);
export const CopyIcon = make(Copy01Icon);
export const DownloadIcon = make(Download04Icon);
export const UploadIcon = make(Upload04Icon);
export const SunIcon = make(Sun01Icon);
export const MoonIcon = make(Moon02Icon);
export const EyeIcon = make(ViewIcon);
export const EyeOffIcon = make(ViewOffSlashIcon);
export const KeyIcon = make(Key01Icon);
export const EditIcon = make(Edit02Icon);
/** A cell's own edit button: a different pencil from the row's Edit (Peek). */
export const EditFieldIcon = make(PencilEdit02Icon);
export const RefreshIcon = make(ArrowReloadHorizontalIcon);
/** A stat's trend against the period before: up, down, or level. */
export const TrendUpIcon = make(ArrowUpRight02Icon, "trend-up");
export const TrendDownIcon = make(ArrowDownRight02Icon, "trend-down");
export const TrendFlatIcon = make(ArrowRight02Icon, "trend-flat");
export const AddIcon = make(PlusSignIcon);
export const SaveViewIcon = make(BookmarkAdd01Icon);
/** Zoom out, beside AddIcon's zoom in. */
export const MinusIcon = make(MinusSignIcon);
/** Opens a short list of tips (the flow canvas's "?"). */
export const HelpCircleIcon = make(HugeHelpCircleIcon);
export const FolderIcon = make(Folder01Icon);
export const InfoIcon = make(InformationCircleIcon);
export const StarOutlineIcon = make(StarIcon);
export const LockedIcon = make(LockIcon);
/** A profile's own password (#127): the row's lock, Set password; Change and Remove beside it. */
export const PasswordIcon = make(SquareLock02Icon);
export const ChangePasswordIcon = make(ResetPasswordIcon);
export const RemovePasswordIcon = make(SquareUnlock02Icon);
export const ChevronDownIcon = make(ArrowDown01Icon);
export const BackIcon = make(ArrowLeft01Icon);
export const DeleteIcon = make(Delete02Icon);
export const GlobeIcon = make(Globe02Icon);
export const ClockIcon = make(Clock01Icon);
export const BuildingIcon = make(Building02Icon);
export const PinIconApp = make(PinIcon);
export const UnpinIconApp = make(PinOffIcon);
export const PlayIcon = make(HugePlayIcon);
export const StopIcon = make(HugeStopIcon);
export const MoreIcon = make(MoreVerticalIcon);
export const SyncIcon = make(Link04Icon);
/** Personal sync across the person's devices (not the window synchronizer's SyncIcon). */
export const SyncedIcon = make(CloudSyncIcon, "synced");
export const SyncOffIcon = make(CloudOffIcon, "sync-off");
/** A conflict copy a sync made: both versions were kept. */
export const SyncConflictIcon = make(CloudAlertIcon, "sync-conflict");
/** Opens a page outside the app (the ProxyShard shop). */
export const ExternalLinkIcon = make(ArrowUpRight01Icon);
export const RestoreIcon = make(ArchiveRestoreIcon);
export const UndoIcon = make(Undo02Icon);
export const ComputerIcon = make(HugeComputerIcon);
export const CheckCircleIcon = make(CheckmarkCircle02Icon);
export const AlertIcon = make(Alert02Icon);
/** A round alert glyph (the kit's own error mapping), not the warning triangle above. */
export const AlertCircleIcon = make(HugeAlertCircleIcon);
export const KeyboardIcon = make(HugeKeyboardIcon);
export const SaveIcon = make(FloppyDiskIcon);
export const FilterIcon = make(HugeFilterIcon);
export const CloseIcon = make(Cancel01Icon);
/** The tick of a Save button that sits beside a field (a cell edit). */
export const TickIcon = make(Tick02Icon);
export const TagIcon = make(Tag01Icon);
export const DisplayIcon = make(SlidersHorizontalIcon);
/** The list header's columns button (owner feature 2026-09-26). */
export const ColumnsIcon = make(ColumnsThreeCogIcon);
export const MoveLeftIcon = make(ArrowLeft01Icon);
export const MoveRightIcon = make(ArrowRight01Icon);
/** The sort mark after a column title. */
export const SortAscIcon = make(ArrowUp02Icon);
export const SortDescIcon = make(ArrowDown02Icon);
export const PauseIcon = make(HugePauseIcon);
export const RenameIcon = make(PencilEdit01Icon);
/** Open in Peek: the panel at the right (E2, CRAFT-10). */
export const PeekIcon = make(SidebarRight01Icon);
/** Copy ID: not Clone's copy glyph (CRAFT-10). */
export const IdCardIcon = make(IdIcon);
export const RegenerateIcon = make(MagicWand01Icon);
export const FolderMinusIconApp = make(FolderMinusIcon);
export const FolderAddIconApp = make(FolderAddIcon, "folder-add");
export const ChevronRightIcon = make(ArrowRight01Icon);
/** Work in progress (Starting, Stopping, Testing): the glyph the kit
 *  Button's spinner draws, so the app has one spinner. Turn it with
 *  `animate-spin`; it keeps turning under Reduce motion (ANIM-05). */
export const SpinnerIcon = make(Loading03Icon, "spinner");

export function ShardLogo({ size = 24, className }: { size?: number; className?: string }) {
  const height = Math.round((size * 18) / 20);
  return (
    <svg width={size} height={height} viewBox="0 0 20 18" fill="none" xmlns="http://www.w3.org/2000/svg" className={className} aria-hidden="true">
      {/* L Protective Shield Stance */}
      <path
        d="M1 2C1 1.44772 1.44772 1 2 1H4.5C5.05228 1 5.5 1.44772 5.5 2V12.5H9C9.55228 12.5 10 12.9477 10 13.5V15.5C10 16.0523 9.55228 16.5 9 16.5H2C1.44772 16.5 1 16.0523 1 15.5V2Z"
        fill="currentColor"
      />
      {/* P Browser Arc & Stem */}
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M8 2C8 1.44772 8.44772 1 9 1H13.8C16.948 1 19.5 3.552 19.5 6.7C19.5 9.848 16.948 12.4 13.8 12.4H11.5V15.5C11.5 16.0523 11.0523 16.5 10.5 16.5H8.5C7.94772 16.5 7.5 16.0523 7.5 15.5V2H8ZM11.5 4V9.4H13.8C15.291 9.4 16.5 8.191 16.5 6.7C16.5 5.209 15.291 4 13.8 4H11.5Z"
        fill="currentColor"
      />
      {/* Antidetect Radar Iris */}
      <circle cx="13.8" cy="6.7" r="1.5" fill="currentColor" />
    </svg>
  );
}

export function ShardMini({ size = 18, className }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg" className={className} aria-hidden="true">
      {/* L Mini */}
      <path
        d="M1 1.5C1 1.22 1.22 1 1.5 1H3.5C3.78 1 4 1.22 4 1.5V11H7.5C7.78 11 8 11.22 8 11.5V13.5C8 13.78 7.78 14 7.5 14H1.5C1.22 14 1 13.78 1 13.5V1.5Z"
        fill="currentColor"
      />
      {/* P Mini */}
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M6.5 1.5C6.5 1.22 6.72 1 7 1H11.5C13.985 1 16 3.015 16 5.5C16 7.985 13.985 10 11.5 10H9.5V13.5C9.5 13.78 9.28 14 9 14H7C6.72 14 6.5 13.78 6.5 13.5V1.5ZM9.5 3.5V7.5H11.5C12.605 7.5 13.5 6.605 13.5 5.5C13.5 4.395 12.605 3.5 11.5 3.5H9.5Z"
        fill="currentColor"
      />
      {/* Iris */}
      <circle cx="11.5" cy="5.5" r="1.2" fill="currentColor" />
    </svg>
  );
}

export function GithubMark({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="currentColor" aria-hidden>
      <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z" />
    </svg>
  );
}

/** Folder icons a user can pick (Phase 2C). Keyed by the id stored with the Folder. */
export const FOLDER_ICON_GLYPHS = {
  folder: make(Folder01Icon),
  briefcase: make(Briefcase01Icon),
  cart: make(ShoppingCart01Icon),
  megaphone: make(Megaphone01Icon),
  people: make(UserGroupIcon),
  globe: make(Globe02Icon),
  star: make(StarIcon),
  code: make(CodeIcon),
  game: make(GameController01Icon),
  wallet: make(Wallet01Icon),
  mail: make(Mail01Icon),
  rocket: make(Rocket01Icon),
};

/** A step's glyph (see blockIcon in the automation entity): decoration only. */
export function StepIcon({ icon, className, ...rest }: IconProps & { icon: HugeiconsIconProps["icon"] }) {
  return (
    <HugeiconsIcon
      icon={icon}
      strokeWidth={ICON_STROKE}
      data-step-icon=""
      aria-hidden="true"
      focusable="false"
      className={cn(!rest.size && !hasExplicitSize(className) && "size-[17px]", className)}
      {...rest}
    />
  );
}
