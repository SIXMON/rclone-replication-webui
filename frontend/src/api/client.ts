import axios from 'axios';

const client = axios.create({
  baseURL: '/api',
});

client.interceptors.response.use(
  (response) => response,
  (error) => {
    const message = error.response?.data?.error || error.message || 'Unknown error';
    return Promise.reject(new Error(message));
  }
);

export default client;
