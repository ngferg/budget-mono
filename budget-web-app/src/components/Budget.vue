<script setup>
import { ref, onMounted, nextTick } from 'vue'
import { store } from '../store.js'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';

const error = ref("");
const budget = ref(null);
const categories = ref(null);
const year = ref(new Date().getFullYear());
const month = ref(new Date().getMonth() + 1);
const item_descriptions = ref([]);
const item_amounts = ref([]);
const show_edit_modal = ref(false);
const edit_item_id = ref(null);
const edit_item_description = ref("");
const edit_item_amount = ref("");
const edit_description_input = ref(null);
const last_month_clonable = ref(false);

const get_budget = async () => {
  console.log("Get budget for: " + store.get_email());
  try {
    const resp = await fetch(API_BASE_URL + '/users/budget', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': store.get_token(),
      },
      body: JSON.stringify({
        'email': store.get_email(),
        'year': year.value,
        'month': month.value,
      })
    });
    if (resp.status === 200) {
      const j = await resp.json();
      budget.value = j.budget;
      categories.value = j.categories;
      item_amounts.value = new Array(categories.value.length).fill('');
      item_descriptions.value = new Array(categories.value.length).fill('');
      last_month_clonable.value = j.last_month_clonable;
    } else {
      error.value = "Error: " + resp.status;
      if (resp.status === 401) {
        store.log_out();
      }
    }
  } catch (e) {
    error.value = "Error: " + e.message;
  }
};

onMounted(async () => {
  await get_budget();
})

const formatCents = (cents) => {
  if (typeof cents !== 'number' || Number.isNaN(cents)) return '$0.00';
  return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(cents / 100);
};

const delete_line_item = async (item_id) => {
  try {
    const resp = await fetch(API_BASE_URL + '/users/budget/line_item', {
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': store.get_token(),
      },
      body: JSON.stringify({
        'email': store.get_email(),
        'year': year.value,
        'month': month.value,
        'item_id': item_id,
      })
    });
    if (resp.status === 204) {
      await get_budget();
    } else {
      error.value = "Error: " + resp.status;
    }
  } catch (e) {
    error.value = "Error: " + e.message;
  }
}

const new_line_item = async (category_id) => {
  if (item_descriptions.value[category_id] === '' || item_amounts.value[category_id] === '') {
    console.error("Error: Description and Amount are required.");
    return;
  }
  if (isNaN(item_amounts.value[category_id]) || item_amounts.value[category_id] <= 0) {
    console.error("Error: Amount must be a positive number.");
    return;
  }
  try {
    const resp = await fetch(API_BASE_URL + '/users/budget/line_item', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': store.get_token(),
      },
      body: JSON.stringify({
        'email': store.get_email(),
        'year': year.value,
        'month': month.value,
        'category_id': category_id,
        'description': item_descriptions.value[category_id],
        'amount': item_amounts.value[category_id] * 100,
      })
    });
    if (resp.status === 201) {
      item_descriptions.value[category_id] = '';
      item_amounts.value[category_id] = '';
      await get_budget();
    } else {
      error.value = "Error: " + resp.status;
    }
  } catch (e) {
    error.value = "Error: " + e.message;
  }
};

const open_edit_line_item_modal = (item_id, description, amount) => {
  edit_item_id.value = item_id;
  edit_item_description.value = description;
  edit_item_amount.value = amount / 100;
  show_edit_modal.value = true;
  nextTick(() => {
    edit_description_input.value?.focus();
  });
};

const close_edit_modal = () => {
  show_edit_modal.value = false;
};

const save_edit_line_item = async () => {
  if (edit_item_description.value === '' || edit_item_amount.value === '') {
    console.error("Error: Description and Amount are required.");
    return;
  }
  if (isNaN(edit_item_amount.value) || edit_item_amount.value <= 0) {
    console.error("Error: Amount must be a positive number.");
    return;
  }
  try {
    const resp = await fetch(API_BASE_URL + '/users/budget/line_item', {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': store.get_token(),
      },
      body: JSON.stringify({
        'email': store.get_email(),
        'item_id': edit_item_id.value,
        'description': edit_item_description.value,
        'amount': edit_item_amount.value * 100,
      })
    });
    if (resp.status === 200) {
      close_edit_modal();
      await get_budget();
    } else {
      error.value = "Error: " + resp.status;
    }
  } catch (e) {
    error.value = "Error: " + e.message;
  }
};

const clone_last_month = async () => {
  let source_year = year.value;
  let source_month = month.value - 1;
  if (source_month === 0) {
    source_month = 12;
    source_year -= 1;
  }
  try {
    const resp = await fetch(API_BASE_URL + '/users/budget/clone_month', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': store.get_token(),
      },
      body: JSON.stringify({
        'email': store.get_email(),
        'source_year': source_year,
        'source_month': source_month,
        'target_year': year.value,
        'target_month': month.value,
      })
    });
    if (resp.status === 201) {
      await get_budget();
    } else {
      error.value = "Error: " + resp.status;
    }
  } catch (e) {
    error.value = "Error: " + e.message;
  }
}

async function last_month() {
  if (month.value === 1) {
    month.value = 12;
    year.value -= 1;
  } else {
    month.value -= 1;
  }
  await get_budget();
}

async function next_month() {
  if (month.value === 12) {
    month.value = 1;
    year.value += 1;
  } else {
    month.value += 1;
  }
  await get_budget();
}
</script>

<template>
  <div class="card" v-if="error">
    <p class="error-text">{{ error }}</p>
  </div>

  <div v-if="budget !== null" class="card">
    <h2><button @click="last_month">&lt;</button>Budget for {{ month }}/{{ year }}<button
        @click="next_month">&gt;</button></h2>
    <div v-if="last_month_clonable">
      <button @click="clone_last_month">Clone Last Month's Budget</button>
    </div>
    <h3 class="text-2xl font-bold text-emerald-300 mt-6 mb-4 border-b-2 border-emerald-500 pb-2">Overview:</h3>
    <h4 class="text-lg font-semibold text-emerald-200 mb-2 pl-4">
      Income: {{formatCents(budget[categories[0]?.id]?.map(item => item.amount).reduce((a, c) => a + c, 0) || 0)}}
    </h4>
    <h4 class="text-lg font-semibold text-emerald-200 mb-2 pl-4">
      Expenses: {{formatCents(categories.slice(1).flatMap(cat => budget[cat.id] || []).reduce((sum, item) => sum +
        item.amount, 0))}}
    </h4>
    <h3 class="text-2xl font-bold text-emerald-300 mt-8 mb-4 border-b-2 border-emerald-500 pb-2">Categories:</h3>
    <div>
      <h4 v-for="category in categories" :key="category.id" class="text-xl font-bold text-emerald-100 mt-6 mb-3">
        {{ category.name }}: {{formatCents(budget[category.id].map(item => item.amount).reduce((a, c) => a + c, 0))}}
        <ul>
          <li v-for="item in budget[category.id]" :key="item.id">
            {{ item.description }}: {{ formatCents(item.amount) }} <button
              @click="delete_line_item(item.id)">-</button><button
              @click="open_edit_line_item_modal(item.id, item.description, item.amount)">✎</button>
          </li>
          <li><input type="text" placeholder="Add new line item" v-model="item_descriptions[category.id]"></input>:
            <input type="number" placeholder="Amount" v-model.number="item_amounts[category.id]"
              @keydown.enter="new_line_item(category.id)"></input><button
              @click=" new_line_item(category.id)">+</button>
          </li>

        </ul>
      </h4>
    </div>
  </div>

  <Teleport to="body">
    <!-- backdrop -->
    <div v-if="show_edit_modal" class="modal-backdrop" @click="close_edit_modal"></div>

    <!-- modal content -->
    <div v-if="show_edit_modal" class="modal-container">
      <div class="modal-header">
        <h3>Edit Line Item</h3>
        <button @click="close_edit_modal" class="close-button">&times;</button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label>Description:</label>
          <input type="text" v-model="edit_item_description" ref="edit_description_input"
            @keydown.enter="save_edit_line_item" />
        </div>
        <div class="form-group">
          <label>Amount:</label>
          <input type="number" v-model.number="edit_item_amount" step="0.01" @keydown.enter="save_edit_line_item" />
        </div>
      </div>
      <div class="modal-footer">
        <button @click="save_edit_line_item">Save</button>
        <button @click="close_edit_modal">Cancel</button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.error-text {
  color: #ff6b6b;
  font-weight: 500;
}

h1,
h3,
h4,
h5,
h6 {
  text-align: left;
}

ul {
  list-style: none;
  padding: 0;
  margin: 0.8em 0;
}

li {
  text-align: left;
  padding: 0.75em 1em;
  margin: 0.5em 0;
  background: rgba(16, 185, 129, 0.08);
  border-radius: 8px;
  border-left: 3px solid #10b981;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
}

li:hover {
  background: rgba(16, 185, 129, 0.12);
  border-left-color: #34d399;
  transform: translateX(2px);
}

li:last-child {
  background: rgba(16, 185, 129, 0.03);
  border-left-color: #6b7280;
  border-style: dashed;
}

li:last-child:hover {
  background: rgba(16, 185, 129, 0.08);
  border-left-color: #10b981;
}

h2 {
  color: #d1fae5;
  font-size: 1.5em;
  margin-bottom: 1.5em;
  display: flex;
  align-items: center;
  gap: 1em;
}

h2 button {
  padding: 0.4em 0.8em;
  font-size: 0.8em;
  min-width: 40px;
}


input {
  border: none;
  border-bottom: 2px solid #10b981;
  background-color: rgba(16, 185, 129, 0.05);
  padding: 8px 8px;
  font-size: 1em;
  color: #ffffff;
  border-radius: 4px 4px 0 0;
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

.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  z-index: 10;
}

.modal-container {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: linear-gradient(135deg, #1a3a2a 0%, #0f2818 100%);
  padding: 30px;
  border-radius: 12px;
  border: 1px solid #10b981;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3), inset 0 0 1px rgba(16, 185, 129, 0.2);
  z-index: 20;
  max-width: 500px;
  width: 90%;
  color: #fff;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 2px solid #10b981;
  padding-bottom: 15px;
  margin-bottom: 20px;
}

.modal-header h3 {
  margin: 0;
  color: #d1fae5;
  font-size: 1.3em;
}

.modal-body {
  margin-bottom: 25px;
}

.form-group {
  margin-bottom: 20px;
  text-align: left;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  color: #a7f3d0;
  font-size: 0.95em;
}

.form-group input {
  width: 100%;
  box-sizing: border-box;
}

.modal-footer {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
}

.modal-footer button:first-child {
  background-color: #10b981;
}

.modal-footer button:last-child {
  background-color: transparent;
  border: 2px solid #6b7280;
  color: #9ca3af;
}

.modal-footer button:last-child:hover {
  background-color: transparent;
  border-color: #10b981;
  color: #10b981;
}

.close-button {
  background: none;
  border: none;
  font-size: 1.8rem;
  cursor: pointer;
  padding: 0;
  color: #9ca3af;
  transition: color 0.2s ease;
}

.close-button:hover {
  color: #10b981;
}
</style>