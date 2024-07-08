const { invoke } = window.__TAURI__.tauri;

let ch_cnt;

let gauges = {
  types: [],
  details: [],
};

let power = false;

window.addEventListener("DOMContentLoaded", async () => {
  await init_ui();
  await init_event_handler();
});

async function init_ui () {
  ch_cnt = await invoke("get_channel_count", {});

  document.querySelector("div#channel-conf-area").innerHTML += build_channel_conf(0);

  for (let i = 1; i <= ch_cnt; i++) {
    document.querySelector("table#channel-list tr").innerHTML += build_channel_btn(i);
    document.querySelector("div#channel-conf-area").innerHTML += build_channel_conf(i);
  }

  [ gauges.types, gauges.details] = await invoke("get_gauge_types", {});
  let gauge_types = "<option disabled selected value=''>Select gauge type</option>";
  
  for (const [i, item] of gauges.details.entries()) {
    gauge_types += `<option value='${gauges.types[i]}'>${item}</option>`;
  };

  document.querySelectorAll("select.channel-type").forEach(x => {
    x.innerHTML = gauge_types;
  });

  document.querySelector("#device").innerHTML = await build_port_selection();
}

async function init_event_handler() {
  document.querySelector("#refresh-port").addEventListener("click", async e => {
    document.querySelector("#device").innerHTML = await build_port_selection();
  });

  document.querySelector("#open-config-dir").addEventListener("click", async e => {
    await invoke("open_config_dir", {});
  });

  const powerbtn = document.querySelector("button#power");

  powerbtn.addEventListener("click", async e => {
    power = !power;
    powerbtn.classList.toggle("active");
    powerbtn.innerText = powerbtn.innerText === "ACTIVATE" ? "DEACTIVATE" : "ACTIVATE";
    await config_event_handler(e);
  });

  /* .config change event */
  document.querySelector(".container").addEventListener("change", async e => {
    await config_event_handler(e);
  });

  document.querySelectorAll("td.channel-btn").forEach(x => x.addEventListener("click", e => {
    document.querySelectorAll("td.channel-btn.selected").forEach(x => x.classList.remove('selected'));
    e.srcElement.classList.add("selected");

    document.querySelectorAll("div.channel-conf:not(.hidden)").forEach(d => d.classList.add('hidden'));
    document.querySelector(`#channel-conf-${e.srcElement.id}`).classList.remove("hidden");
  }));
}

async function config_event_handler(e) {
  if (!(e.target && e.target.matches('.config'))) {
    return;
  }

  if (e.target.matches('.channel-active')) {
    if (e.target.checked) {
      const target = e.target.closest('div.channel-conf').id.replace("channel-conf-", '');
      document.querySelector(`#${target}`).classList.add('active');
    }
  }

  // set channel detailed configuration ui
  if (e.target.matches('.channel-type')) {
    e.target.closest('table').querySelector('td.channel-detail').innerHTML = await build_channel_detail(e.target.value);
  }

  // config validation
  if (e.target.closest('div.channel-conf')) {
    // channel-conf section event
    if (!validate_channel_config(e.target.closest('div.channel-conf').id)) {
      return;
    }
  }

  let config = validate_ui_config();

  if (config) {
    await invoke("config", config);
  }
}

function validate_ui_config() {
  let port = document.querySelector('#device').value;
  let update = document.querySelector('#interval').value;
  
  if (!port || !update) {
    return null;
  }

  let active_gauges = [];

  for (let i = 1; i <= ch_cnt; i++) {
    if (!document.querySelector(`div#channel-conf-ch${i} input.channel-active`).checked) {
      continue;
    }

    let type = document.querySelector(`div#channel-conf-ch${i} select.channel-type`).value;
    let gauge = {};

    switch (type) {
      case "CpuUsage":
      case "CpuFreq":
        gauge[type] = {
          core: document.querySelector(`div#channel-conf-ch${i} select.channel-coreid`).value
        };
        break;

      case "NetTx":
      case "NetRx":
      case "NetTxRx":
        gauge[type] = {
          netif: document.querySelector(`div#channel-conf-ch${i} select.channel-netif`).value,
          unit: document.querySelector(`div#channel-conf-ch${i} select.channel-speed`).value,
        };
        break;

      default:
        break;
    }

    active_gauges.push(gauge);
  }

  return {
    power: power,
    port: port,
    update: update,
    active: active_gauges,
  }
}

function validate_channel_config(id) {
  const target = document.querySelector(`#${id}`);

  /* enable active checkbox if others are valid */
  switch (target.querySelector(`select.channel-type`).value) {
    case "CpuUsage":
    case "CpuFreq":
      if (target.querySelector(`select.channel-coreid`).value) {
        target.querySelector(`input.channel-active`).disabled = false;
      } else {
        target.querySelector(`input.channel-active`).disabled = true;
        return false;
      }
      break;

    case "NetTx":
    case "NetRx":
    case "NetTxRx":
      if (target.querySelector(`select.channel-netif`).value && target.querySelector(`select.channel-speed`).value) {
        target.querySelector(`input.channel-active`).disabled = false;
      } else {
        target.querySelector(`input.channel-active`).disabled = true;
        return false;
      }
      break;

    default:
      target.querySelector(`input.channel-active`).disabled = false;
      break;
  }

  if (!target.querySelector(`input.channel-active`).checked) {
    return false;
  }

  return true;
}

async function build_port_selection() {
  let html = "<option disabled selected value=''>Select serial port</option>";

  const ports = await invoke("get_ports", {});

  for (let p of ports) {
    html += `<option value='${p[0]}'>${p[1]}</option>`;
  }

  return html;
}

function build_channel_btn(ch) {
  return `<td id="ch${ch}" class="channel-btn">CH ${ch}</td>`;
}

function build_channel_conf(ch) {
  return `
    <div id="channel-conf-${ch ? 'ch' + ch : 'none'}" class="channel-conf ${ch ? 'hidden' : ''}">
      <table>
        <colgroup>
          <col width="30%">
          <col width="70%">
        </colgroup>
        <tr>
          <th>Active</th>
          <td><input disabled type="checkbox" class="channel-active config"></td>
        </tr>
          <th>Type</th>
          <td><select ${ch ? '' : 'disabled'} class="channel-type config"></select></td>
        </tr>
        <tr>
          <th>Config</th>
          <td class="channel-detail">-</td>
        </tr>
      </table>
    </div>`;
}

function build_channel_detail(value) {
  switch (value) {
    case "CpuUsage":
    case "CpuFreq":
      return build_core_selection();

    case "NetTx":
    case "NetRx":
    case "NetTxRx":
      return build_network_selection();

    default:
      return "-";
  }
}

async function build_core_selection() {
  const core_cnt = await invoke("get_core_count", {});

  let html = `<select class='channel-coreid config'>
                <option disabled selected value=''>Select Core</option>
                <option value='-1'>All Core</option>`;

  for (let i = 0; i < core_cnt; i++) {
    html += `<option value='${i}'>Core ${i}</option>`;
  }

  html += `</select>`;

  return html;
}

async function build_network_selection() {
  const netif_list = await invoke("get_netifs", {});
  const speed_unit_list = await invoke("get_speed_units", {});

  let html = `<select class='channel-netif config'>
                <option disabled selected value=''>Select Network</option>`;

  for (let i = 0; i < netif_list.length; i++) {
    html += `<option value='${netif_list[i]}'>${netif_list[i]}</option>`;
  }

  html += `</select><select class='channel-speed config'>`;
  html += `<option disabled selected value=''>Select Speed Unit</option>`;

  for (let i = 0; i < speed_unit_list.length; i++) {
    html += `<option value='${speed_unit_list[i]}'>${speed_unit_list[i]}</option>`;
  }

  html += `</select>`;

  return html;
}
