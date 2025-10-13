import { usePostQuery } from '@/services/aggregator';

export const App = () => {    
  const { data: posts, isLoading, error } = usePostQuery('posts', { skip: false,  keepUnusedDataFor: 0 });
    
  if (isLoading) {
    return <div>Connecting to the post stream...</div>;
  }
  if (error) {
    const errorMessage = 'error' in error ? error.error : JSON.stringify(error);
    return <div>Error connecting to stream: {errorMessage}</div>;
  }

    console.log(posts);

  return (
    <div>
      <h1>Live Post Stream</h1>
      <ul>
        {posts?.map((post, index) => (
          <li key={index}>{post}</li>
        ))}
      </ul>
      {(!posts || posts.length === 0) && <p>Waiting for the first post...</p>}
    </div>
  );
};


export default App;
