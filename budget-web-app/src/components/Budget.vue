<script setup>
import { ref, onMounted } from 'vue'
import { store } from '../store.js'
const error = ref("");
const budget = ref(null);
const categories = ref(null);
const year = ref(new Date().getFullYear());
const month = ref(new Date().getMonth() + 1);
const item_descriptions = ref([]);
const item_amounts = ref([]);

const get_budget = async () => {
  console.log("Get budget for: " + store.get_email());
  try {
    const resp = await fetch('/api/users/budget', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
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
    } else {
      error.value = "Error: " + resp.status;
    }
  } catch (e) {
    error.value = "Error: " + resp.status;
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
    const resp = await fetch('/api/users/budget/line_item', {
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/json',
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
    error.value = "Error: " + resp.status;
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
    const resp = await fetch('/api/users/budget/line_item', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
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
    error.value = "Error: " + resp.status;
  }
};
</script>

<template>
  <h1>Budget Page</h1>

  <div class="card" v-if="error">
    <p class="error-text">{{ error }}</p>
  </div>

  <div v-if="budget !== null" class="card">
    <h2>Budget for {{ month }}/{{ year }}</h2>
    <h3>Categories:</h3>
    <div>
      <h4 v-for="category in categories" :key="category.id">
        {{ category.name }}: {{formatCents(budget[category.id].map(item => item.amount).reduce((a, c) => a + c, 0))}}
        <ul>
          <li v-for="item in budget[category.id]" :key="item.id">
            {{ item.description }}: {{ formatCents(item.amount) }} <button @click="delete_line_item(item.id)">-</button>
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

</template>

<style scoped>
.error-text {
  color: #AA0000;
}

h1,
h2,
h3,
h4,
h5,
h6,
li {
  text-align: left;
}

input {
  border: none;
  border-bottom: 2px solid #333;
  background-color: transparent;
  padding: 4px 0;
  font-size: 1em;
}

input:focus {
  outline: none;
  border-bottom-color: #0066cc;
}

input::placeholder {
  color: #999;
}
</style>
