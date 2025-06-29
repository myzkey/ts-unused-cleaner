import React from 'react';

interface CardProps {
  title: string;
  content: string;
  footer?: React.ReactNode;
}

export const Card: React.FC<CardProps> = ({ title, content, footer }) => {
  return (
    <div className="card">
      <div className="card-header">
        <h3>{title}</h3>
      </div>
      <div className="card-body">
        <p>{content}</p>
      </div>
      {footer && (
        <div className="card-footer">
          {footer}
        </div>
      )}
    </div>
  );
};

export default Card;