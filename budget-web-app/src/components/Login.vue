<script setup>
import { ref, nextTick } from 'vue'
import { store } from '../store.js'

const AUTH_BASE_URL = import.meta.env.VITE_AUTH_BASE_URL || 'http://localhost:3001';
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';
const email = ref(store.get_email());
const error = ref("");
const code = ref("");
const token = ref(store.get_token());
const code_requested = ref(false);
const code_input = ref(null);

const request_code = async () => {
  console.log("Request code for: " + email.value);
  code_requested.value = true;
  nextTick(() => {
    code_input.value.focus();
  });
  try {
    const resp = await fetch(AUTH_BASE_URL + '/request_code', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        'email': email.value,
      })
    });
    if (resp.status !== 200) {
      error.value = "Error: " + resp.status;
      store.log_out();
    }
  } catch (e) {
    error.value = "Error: " + e.message;
    store.log_out();
  }
};

const verify_code = async () => {
  console.log("Verify code for: " + email.value);
  try {
    const resp = await fetch(AUTH_BASE_URL + '/verify_code', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        'email': email.value,
        'code': code.value,
      })
    });
    console.log(resp);
    if (resp.status === 200) {
      const data = await resp.json();
      token.value = data.token;
      await login();
    } else {
      error.value = "Incorrect code";
      store.log_out();
    }
  } catch (e) {
    error.value = "Error: " + e.message;
    store.log_out();
  }
};

const login = async () => {
  console.log("Log in as: " + email.value);
  try {
    const resp = await fetch(API_BASE_URL + '/users', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': token.value,
      },
      body: JSON.stringify({
        'email': email.value,
      })
    });
    if (resp.status === 201 || resp.status === 409) {
      store.log_in_as(email.value, token.value);
    } else {
      error.value = "Login failed";
      store.log_out();
    }
  } catch (e) {
    error.value = "Error: " + e.message;
    store.log_out();
  }
};
</script>

<template>
  <div class="card login-card">
    <div><input type="text" placeholder="email" v-model.trim="email" @keyup.enter="request_code" /></div>
    <div><input type="text" placeholder="code" v-model.trim="code" @keyup.enter="verify_code" v-if="code_requested"
        ref="code_input" />
    </div>
    <p class="error-text">{{ error }}</p>
  </div>

</template>

<style scoped>
.error-text {
  color: #ff6b6b;
  font-weight: 500;
}

input {
  border: none;
  border-bottom: 2px solid #10b981;
  background-color: rgba(16, 185, 129, 0.05);
  padding: 10px 8px;
  font-size: 1em;
  color: #ffffff;
  border-radius: 4px 4px 0 0;
  margin-bottom: 1em;
  width: 100%;
  max-width: 300px;
  transition: all 0.2s ease;
}

input:focus {
  outline: none;
  border-bottom-color: #34d399;
  background-color: rgba(16, 185, 129, 0.1);
  box-shadow: 0 2px 8px rgba(16, 185, 129, 0.2);
}

input::placeholder {
  color: #9ca3af;
}
</style>
