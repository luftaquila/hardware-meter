const { invoke } = window.__TAURI__.tauri;

let ch_cnt;
let gauge_types_list;

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

  gauge_types_list = await invoke("get_gauge_types", {});
  let gauge_types = "<option disabled selected>Select gauge type</option>";
  
  for (const [i, item] of gauge_types_list.entries()) {
    gauge_types += `<option value='${i}'>${item}</option>`;
  };

  document.querySelectorAll("select.channel-type").forEach(x => {
    x.innerHTML = gauge_types;
  });

  document.querySelector("#device").innerHTML = await invoke("get_ports", {});
}

async function init_event_handler() {
  document.querySelector("#refresh-port").addEventListener("click", async e => {
    document.querySelector("#device").innerHTML = await invoke("get_ports", {});
  });

  document.querySelector("#open-config-dir").addEventListener("click", async e => {
    await invoke("open_config_dir", {});
  });

  document.querySelector(".container").addEventListener("change", async e => {
    if (e.target && e.target.matches('.config')) {
      // await invoke("config", {});
    }

    if (e.target && e.target.matches('.channel-type')) {
      e.target.closest('table').querySelector('input.channel-active').disabled = false;
      e.target.closest('table').querySelector('td.channel-detail').innerHTML = await build_channel_detail(e.target.value);
    }
  });

  document.querySelectorAll("td.channel-btn").forEach(x => x.addEventListener("click", e => {
    document.querySelectorAll("td.channel-btn.selected").forEach(x => x.classList.remove('selected'));
    e.srcElement.classList.add("selected");

    document.querySelectorAll("div.channel-conf:not(.hidden)").forEach(d => d.classList.add('hidden'));
    document.querySelector(`#channel-conf-${e.srcElement.id}`).classList.remove("hidden");
  }));
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
  const target = gauge_types_list[value];

  switch (gauge_types_list[value]) {
    case "CPU Utilization":
    case "CPU Frequency":
      return build_core_selection();

    case "Network Transmit Speed":
    case "Network Receive Speed":
    case "Network Receive & Transmit Speed":
      return build_network_selection();

    default:
      return "-";
  }
}

async function build_core_selection() {
  const core_cnt = await invoke("get_core_count", {});

  let html = `<select class='channel-coreid'>
                <option disabled selected>Select Core</option>
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

  let html = `<select class='channel-netif'>
                <option disabled selected>Select Network</option>`;

  for (let i = 0; i < netif_list.length; i++) {
    html += `<option value='${netif_list[i]}'>${netif_list[i]}</option>`;
  }

  html += `</select><select class='channel-speed'>`;
  html += `<option disabled selected>Select Speed Unit</option>`;

  for (let i = 0; i < speed_unit_list.length; i++) {
    html += `<option value='${speed_unit_list[i]}'>${speed_unit_list[i]}</option>`;
  }

  html += `</select>`;

  return html;
}
