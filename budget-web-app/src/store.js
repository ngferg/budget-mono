import { reactive } from 'vue';

export const store = reactive({
  is_logged_in: false,
  email: '',
  token: '',
  log_in_as(email, token) {
    this.is_logged_in = true;
    this.email = email;
    this.token = token;
  },
  get_email() {
    return this.email;
  },
  get_token() {
    return this.token;
  }
});
