import { reactive } from 'vue';
import { sha256 } from './hash';

const AUTH_BASE_URL = import.meta.env.VITE_AUTH_BASE_URL || 'http://localhost:3001';
const storedEmail = localStorage.getItem('email') || '';
const storedHashedEmail = localStorage.getItem('hashed_email') || '';
const storedToken = localStorage.getItem('token') || '';

export const store = reactive({
  is_logged_in: !!(storedEmail && storedToken),
  email: storedEmail,
  hashedEmail: storedHashedEmail,
  token: storedToken,
  async log_in_as(email, token) {
    this.is_logged_in = true;
    this.email = email;
    this.hashedEmail = await sha256(email);
    this.token = token;
    localStorage.setItem('email', email);
    localStorage.setItem('hashed_email', this.hashedEmail);
    localStorage.setItem('token', token);
    localStorage.setItem('has_ever_logged_in', 'true');
  },
  async log_out(logout_all = false) {
    try {
      const resp = await fetch(AUTH_BASE_URL + '/logout', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          'hashed_email': this.hashedEmail,
          'token': this.token,
          'logout_all': logout_all,
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
    this.hashedEmail = '';
    this.token = '';
    localStorage.removeItem('email');
    localStorage.removeItem('hashed_email');
    localStorage.removeItem('token');
  },
  get_email() {
    return this.email;
  },
  get_hashed_email() {
    return this.hashedEmail;
  },
  get_token() {
    return this.token;
  },
  has_ever_logged_in() {
    return !!localStorage.getItem('has_ever_logged_in');
  }
});
