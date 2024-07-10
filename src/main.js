const { invoke } = window.__TAURI__.tauri;

let ch_cnt;

let gauges = {
  types: [],
  details: [],
};

let power = false;

window.addEventListener("DOMContentLoaded", async () => {
  await init_ui();
  await load_ui_config();
  await init_event_handler();
});

async function init_ui() {
  ch_cnt = await invoke("get_channel_count", {});

  document.querySelector("div#channel-conf-area").innerHTML += build_channel_conf(0);

  for (let i = 1; i <= ch_cnt; i++) {
    document.querySelector("table#channel-list tr").innerHTML += build_channel_btn(i);
    document.querySelector("div#channel-conf-area").innerHTML += build_channel_conf(i);
  }

  [gauges.types, gauges.details] = await invoke("get_gauge_types", {});
  let gauge_types = "<option disabled selected value=''>Select gauge type</option>";

  for (const [i, item] of gauges.details.entries()) {
    if (item == "Disabled") {
      continue;
    }

    gauge_types += `<option value='${gauges.types[i]}'>${item}</option>`;
  };

  document.querySelectorAll("select.channel-type").forEach(x => {
    x.innerHTML = gauge_types;
  });

  document.querySelector("#device").innerHTML = await build_port_selection();
}

async function load_ui_config() {
  const config = await invoke("get_config", {});

  if (!config) {
    return;
  }

  /* set update rate */
  document.querySelector('select#interval').value = config.update;

  /* set port */
  const select = document.querySelector('select#device');

  for (let i = 0; i < select.options.length; i++) {
    if (select.options[i].value === config.port) {
      select.selectedIndex = i;
      break;
    }
  }

  /* set channel configs */
  for (let i = 0; i < ch_cnt; i++) {
    if (config.gauges[i] === "Disabled") {
      continue;
    }

    document.querySelector(`td#ch${i + 1}`).classList.add('active');

    const target = document.querySelector(`div#channel-conf-ch${i + 1}`);

    const active = target.querySelector('input.channel-active')
    active.disabled = false;
    active.checked = true;

    const current_type = typeof(config.gauges[i]) === "string" ? config.gauges[i] : Object.keys(config.gauges[i])[0];
    const type = target.querySelector('select.channel-type');

    for (let j = 0; j < type.options.length; j++) {
      if (type.options[j].value === current_type) {
        type.selectedIndex = j;
        target.querySelector('td.channel-detail').innerHTML = await build_channel_detail(current_type);

        switch (current_type) {
          case "CpuUsage":
          case "CpuFreq":
            const coreid = target.querySelector('select.channel-coreid');

            for (let k = 0; k < coreid.options.length; k++) {
              if (coreid.options[k].value == config.gauges[i][current_type].core) {
                coreid.selectedIndex = k;
              }
            }
            break;

          case "NetTx":
          case "NetRx":
          case "NetTxRx":
            const netif = target.querySelector('select.channel-netif');
            const speed = target.querySelector('select.channel-speed');

            for (let k = 0; k < netif.options.length; k++) {
              if (netif.options[k].value === config.gauges[i][current_type].netif) {
                netif.selectedIndex = k;
              }
            }

            for (let k = 0; k < speed.options.length; k++) {
              if (speed.options[k].value == config.gauges[i][current_type].unit) {
                speed.selectedIndex = k;
              }
            }
            break;

          default:
            return "-";
        }
      }
    }
  }


  /* set master power button */
  if (config.power && document.querySelector('select#device').value) {
    power = true;

    const powerbtn = document.querySelector("button#power");
    powerbtn.classList.add("active");
    powerbtn.innerText = "DEACTIVATE";
  }
}

async function init_event_handler() {
  document.querySelector("#refresh-port").addEventListener("click", async e => {
    /* deactivate device */
    if (power) {
      power = false;

      const powerbtn = document.querySelector("button#power");
      powerbtn.classList.remove("active");
      powerbtn.innerText = "ACTIVATE";

      let config = validate_ui_config();

      if (config) {
        await invoke("config", { conf: config });
      }
    }

    document.querySelector("#device").innerHTML = await build_port_selection();
  });

  document.querySelector("#open-config-dir").addEventListener("click", async e => {
    await invoke("open_config_dir", {});
  });

  const powerbtn = document.querySelector("button#power");

  powerbtn.addEventListener("click", async e => {
    if (!validate_ui_config()) {
      return;
    }

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
    e.target.classList.add("selected");

    document.querySelectorAll("div.channel-conf:not(.hidden)").forEach(d => d.classList.add('hidden'));
    document.querySelector(`#channel-conf-${e.target.id}`).classList.remove("hidden");
  }));
}

async function config_event_handler(e) {
  if (!(e.target && e.target.matches('.config'))) {
    return;
  }

  if (e.target.matches('.channel-active')) {
    const target = e.target.closest('div.channel-conf').id.replace("channel-conf-", '');
    document.querySelector(`#${target}`).classList.toggle('active');
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
    await invoke("config", { conf: config });
  }
}

function validate_ui_config() {
  let port = document.querySelector('#device').value;
  let update = document.querySelector('#interval').value;

  if (!port || !update) {
    return null;
  }

  let gauges = [];

  for (let i = 1; i <= ch_cnt; i++) {
    if (!document.querySelector(`div#channel-conf-ch${i} input.channel-active`).checked) {
      gauges.push({ Disabled: null });
      continue;
    }

    let type = document.querySelector(`div#channel-conf-ch${i} select.channel-type`).value;
    let gauge = {};

    switch (type) {
      case "CpuUsage":
      case "CpuFreq":
        gauge[type] = {
          core: Number(document.querySelector(`div#channel-conf-ch${i} select.channel-coreid`).value),
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
        gauge[type] = null;
        break;
    }

    gauges.push(gauge);
  }

  return {
    power: power,
    port: port,
    update: Number(update),
    gauges: gauges,
  }
}

async function validate_channel_config(id) {
  const target = document.querySelector(`#${id}`);
  const active = target.querySelector(`input.channel-active`);

  /* enable active checkbox if others are valid */
  switch (target.querySelector(`select.channel-type`).value) {
    case "CpuUsage":
    case "CpuFreq":
      if (target.querySelector(`select.channel-coreid`).value) {
        active.disabled = false;
      } else {
        active.checked = false;
        active.disabled = true;

        let config = validate_ui_config();

        if (config) {
          await invoke("config", { conf: config });
        }
        return false;
      }
      break;

    case "NetTx":
    case "NetRx":
    case "NetTxRx":
      if (target.querySelector(`select.channel-netif`).value && target.querySelector(`select.channel-speed`).value) {
        active.disabled = false;
      } else {
        active.checked = false;
        active.disabled = true;

        let config = validate_ui_config();

        if (config) {
          await invoke("config", { conf: config });
        }
        return false;
      }
      break;

    default:
      active.disabled = false;
      break;
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

  let speed_unit = {
    name: [],
    desc: [],
  };

  [speed_unit.name, speed_unit.desc] = await invoke("get_speed_units", {});

  let html = `<select class='channel-netif config'>
                <option disabled selected value=''>Select Network</option>`;

  for (let i = 0; i < netif_list.length; i++) {
    html += `<option value='${netif_list[i]}'>${netif_list[i]}</option>`;
  }

  html += `</select><select class='channel-speed config'>`;
  html += `<option disabled selected value=''>Select Speed Unit</option>`;

  for (let i = 0; i < speed_unit.name.length; i++) {
    html += `<option value='${speed_unit.name[i]}'>${speed_unit.desc[i]}</option>`;
  }

  html += `</select>`;

  return html;
}
