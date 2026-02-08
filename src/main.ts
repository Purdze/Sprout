import { createApp } from 'vue';
import App from './App.vue';
import './style.css';

import { library } from '@fortawesome/fontawesome-svg-core';
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
import { faCloudflare, faDiscord } from '@fortawesome/free-brands-svg-icons';

library.add(faCloudflare, faDiscord);

createApp(App).component('FontAwesomeIcon', FontAwesomeIcon).mount('#app');
