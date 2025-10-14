import {  useSseQuery } from '@/services/aggregator';
import { Container, CssBaseline, Paper, Typography, List,  ListItemButton,  ListItemText,  } from '@mui/material';

export const App = () => {    
  const { data: posts, isLoading, error, isFetching } = useSseQuery('posts', { skip: false,  keepUnusedDataFor: 0 });
    
  if (isLoading) {
    return <div>Connecting to the post stream...</div>;
  }
  if (error) {
    const errorMessage = 'error' in error ? error.error : JSON.stringify(error);
    return <div>Error connecting to stream: {errorMessage}</div>;
  }

    console.log(posts);

  return (
    <Container maxWidth="sm">
          <CssBaseline />
      <Paper square sx={{ pb: '50px' }}>
        <Typography variant="h5" gutterBottom component="div" sx={{ p: 2, pb: 0 }}>
          Inbox
        </Typography>
        <List sx={{ mb: 2 }}>
          {posts?.map((post, index) => (
            <ListItemButton key={index}>
                <ListItemText primary={post} />
            </ListItemButton>
          ))}
        </List>
      </Paper>
    {(!posts || posts.length === 0) && <p>Waiting for the first post...</p>}
    </Container>
  );
};


export default App;
