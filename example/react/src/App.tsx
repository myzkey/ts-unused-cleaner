import React from 'react';
import { Home } from './pages/Home';
import { fetchUsers, USED_CONSTANT } from './utils/api';
import { Status } from './types/api';

function App() {
  const [status, setStatus] = React.useState<Status>(Status.PENDING);

  React.useEffect(() => {
    setStatus(Status.LOADING);
    fetchUsers()
      .then(() => setStatus(Status.SUCCESS))
      .catch(() => setStatus(Status.ERROR));
  }, []);

  console.log(USED_CONSTANT);

  return (
    <div className="App">
      <Home />
    </div>
  );
}

export default App;