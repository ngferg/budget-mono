import { reactive } from 'vue';

export const store = reactive({
  is_logged_in: false,
  email: '',
  log_in_as(email) {
    this.is_logged_in = true;
    this.email = email;
  },
  get_email() {
    return this.email;
  }
});
