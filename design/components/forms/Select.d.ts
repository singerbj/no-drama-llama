import type { ChangeEventHandler, CSSProperties } from 'react';
/** Native dropdown for a Choice setting. */
export interface SelectProps {
  value?: string;
  onChange?: ChangeEventHandler<HTMLSelectElement>;
  /** [value, label] pairs */
  options?: [string, string][];
  minWidth?: number | string;
  invalid?: boolean;
  style?: CSSProperties;
  [key: string]: unknown;
}
export declare function Select(props: SelectProps): JSX.Element;
