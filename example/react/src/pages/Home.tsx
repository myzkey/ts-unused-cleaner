import React from 'react';
import { Button } from '../components/button';
import { Card } from '../components/card';

export const Home: React.FC = () => {
  const handleClick = () => {
    console.log('Button clicked!');
  };

  return (
    <div className="home">
      <h1>Welcome to Our App</h1>
      <Card
        title="Getting Started"
        content="This is a sample React application to demonstrate unused component detection."
        footer={
          <Button onClick={handleClick} variant="primary">
            Get Started
          </Button>
        }
      />
    </div>
  );
};

export default Home;