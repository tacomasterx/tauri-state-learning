import { invoke } from "@tauri-apps/api/tauri";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;
let clockParragraphEl: HTMLElement | null;
let powerSpanEl: HTMLElement | null;
let timerInputEl: HTMLInputElement | null;
let timerListEl: HTMLElement | null;
const timerListOpen = '<li class="timer-list-item"';
const timerListClose = '</li>';

// System State functions
// Data object equivalent
interface Data {
  unlisten: UnlistenFn | null;
  systemState: any; // You may replace 'any' with a more specific type if known
};

const data: Data = {
  unlisten: null,
  systemState: null
};

interface timerListData {
    timers: Array<clockData>
}

interface clockData {
    hours: number,
    minutes: number,
    seconds: number,
    milliseconds: number,
    id: number
}

interface powerData {
    power: number
}

// BeforeUnmount lifecycle hook equivalent
function beforeUnmount(): void {
  if (data.unlisten) {
    data.unlisten();
  }
}

// Mounted lifecycle hook equivalent
async function mounted(): Promise<void> {
  data.unlisten = await listen('system_state_update', (event) => {
    data.systemState = event.payload;
    updatePower();
    updateClock();
  });
  await invoke('setup');
}

// Commands functions

async function greet() {
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

async function updateClock() {
    clockParragraphEl = document.querySelector("#clock-parragraph");
    const placeholder: clockData = await invoke("get_clock");
    if (clockParragraphEl) {
        clockParragraphEl.textContent = formatClockTimer(placeholder);
    }
}

function formatClockTimer(placeholder: clockData) {
    let hours: string = "" + placeholder.hours;
    let minutes: string = "" + placeholder.minutes;
    let seconds: string = "" + placeholder.seconds;
    let milliseconds: string = "" + placeholder.milliseconds;
    if ( placeholder.hours < 10 ) {
        hours = "0" + placeholder.hours;
    } 
    if ( placeholder.minutes < 10 ) {
        minutes = "0" + placeholder.minutes;
    } 
    if ( placeholder.seconds < 10 ) {
        seconds = "0" + placeholder.seconds;
    } 
    if ( placeholder.milliseconds < 100 ) {
        milliseconds = "0" + placeholder.milliseconds;
    } else if (placeholder.milliseconds < 10) {
        milliseconds = "00" + placeholder.milliseconds;
    }

    let clockTimerText = hours + ":" + minutes + ":" + seconds + ":" + milliseconds;
    return(clockTimerText);
}

async function updatePower() {
    powerSpanEl = document.querySelector("#power-span");
    let power: powerData = await invoke("get_power");
    if (power && powerSpanEl) {
        powerSpanEl.innerHTML = "" + power.power;
    }
}

async function getTimerList(): Promise<timerListData> {
    let timerListResponse: timerListData = await invoke("get_timer_list");
    return(timerListResponse);
}

// async function updateTimer(index: Number) {
// }
//
// async function addTimerToList() {
// }

async function spawnTimer() {
    if (timerInputEl && timerInputEl.value !== "") {
        let seconds = parseInt( timerInputEl.value );
        let timerResponse: clockData = await invoke("push_timer", {seconds: seconds});
        if (timerResponse) {
            let timerString = formatClockTimer(timerResponse);
            let timerListOpenPlusId = timerListOpen + ' id="timer_' + timerResponse.id + '" >';
            let timerHTML = timerListOpenPlusId + timerString + timerListClose;
            if (timerListEl) {
                let timerListText = timerListEl?.innerHTML;
                timerListEl.innerHTML = timerListText + timerHTML;
            }
        }
    }
}

async function printTimerList(timerList: timerListData) {
    if (timerListEl) {
        let timerUlElement: String = '';
        timerList.timers.forEach( (timer) => {
            let timerString = formatClockTimer(timer);
            let timerListOpenPlusId = timerListOpen + ' id="' + timer.id + '" >';
            let timerHTML = timerListOpenPlusId + timerString + timerListClose;
            timerUlElement = timerUlElement + timerHTML;
        });
        timerListEl.innerHTML = '' + timerUlElement;
    }
}

// Event listeners
// Simulate DOMContentLoaded event for demonstration purposes
document.addEventListener('DOMContentLoaded', async () => {
  await printTimerList( await getTimerList() );
  // Mount the component
  await mounted();
  // Add event listener for beforeunload event to trigger beforeUnmount
  window.addEventListener('beforeunload', beforeUnmount);
});

//
window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  timerInputEl = document.querySelector("#timer-input");
  timerListEl = document.querySelector("#timer-list");
  document.querySelector("#timer-form")?.addEventListener("submit", async (e) => {
      e.preventDefault();
          spawnTimer();
  });
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});
