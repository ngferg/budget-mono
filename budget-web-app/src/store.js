import { reactive } from 'vue';

const AUTH_BASE_URL = import.meta.env.VITE_AUTH_BASE_URL || 'http://localhost:3001';
const storedEmail = localStorage.getItem('email') || '';
const storedToken = localStorage.getItem('token') || '';

export const store = reactive({
  is_logged_in: !!(storedEmail && storedToken),
  email: storedEmail,
  token: storedToken,
  log_in_as(email, token) {
    this.is_logged_in = true;
    this.email = email;
    this.token = token;
    localStorage.setItem('email', email);
    localStorage.setItem('token', token);
  },
  async log_out() {
    try {
      const resp = await fetch(AUTH_BASE_URL + '/logout', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          'email': this.email,
          'token': this.token,
        })
      });
      if (resp.status !== 200) {
        console.error('Error logging out:', e);
      }
    } catch (e) {
      console.error('Error logging out:', e);
    }
    this.is_logged_in = false;
    this.email = '';
    this.token = '';
    localStorage.removeItem('email');
    localStorage.removeItem('token');
  },
  get_email() {
    return this.email;
  },
  get_token() {
    return this.token;
  }
});
