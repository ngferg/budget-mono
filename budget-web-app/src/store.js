import { reactive } from 'vue';

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
  log_out() {
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
