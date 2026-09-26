import type { ReactNode } from 'react';
/**
 * A settings row: label + hint, control on the right.
 * @startingPoint section="Forms" subtitle="Settings rows with every field kind" viewport="700x420"
 */
export interface SettingRowProps {
  label: ReactNode;
  /** muted 12.5px helper line under the label */
  hint?: ReactNode;
  /** control drops below the label (text fields, lists) */
  stacked?: boolean;
  htmlFor?: string;
  /** hide the bottom divider (last row in a Fieldset) */
  last?: boolean;
  children?: ReactNode;
}
export declare function SettingRow(props: SettingRowProps): JSX.Element;
