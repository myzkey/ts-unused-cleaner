import React from 'react';

interface SpinnerProps {
  size?: 'small' | 'medium' | 'large';
  color?: string;
}

export function Spinner({ size = 'medium', color = '#007bff' }: SpinnerProps) {
  const sizeClass = `spinner-${size}`;
  
  return (
    <div 
      className={`spinner ${sizeClass}`}
      style={{ borderTopColor: color }}
    />
  );
}

export default Spinner;