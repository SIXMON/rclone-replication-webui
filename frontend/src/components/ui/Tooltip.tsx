import { ReactNode } from 'react';

interface Props {
  content: string;
  children: ReactNode;
  position?: 'top' | 'bottom' | 'left' | 'right';
}

const positionClasses = {
  top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
  bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
  left: 'right-full top-1/2 -translate-y-1/2 mr-2',
  right: 'left-full top-1/2 -translate-y-1/2 ml-2',
};

export function Tooltip({ content, children, position = 'top' }: Props) {
  return (
    <span className="tooltip-trigger inline-flex">
      {children}
      <span className={`tooltip-content ${positionClasses[position]}`}>
        {content}
      </span>
    </span>
  );
}
