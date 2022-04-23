        <style>
.block {
    position: absolute;
    display: block;
    width: 32px;
    height: 32px;
}

:global(.block-orange) {
    background: hsl(24, 95%, 60%); }
:global(.user-orange) { color: hsl(24, 95%, 60%) ;}
:global(.block-green) {
    background: hsl(100, 85%, 60%);  }
:global(.user-green) { color: hsl(100, 85%, 60%) ;}
:global(.block-blue) {
    background: hsl(200, 95%, 60%);  }
:global(.user-blue) {
    color: hsl(200, 95%, 60%);  }
:global(.block-purple) {
    background: hsl(270, 95%, 60%); }
:global(.user-purple) {
    color: hsl(270, 95%, 60%);  }
.block-wall {
    background: #aaa; }
#blocks {
    position: absolute;
    top: 0; left: 0; width: 100%; height: 100%;
    overflow: hidden;}

#user-list, #block-factory {
    padding: 8px;
    padding-top: 16px;
    background: hsl(250, 10%, 30%);
    color: #fff;

    position: fixed;
    top: 16px;
    left: 16px;
    width: 240px;
    height: auto;
}
.user {
    padding: 8px;
    display: block; }
.user-owner::before {
    content: "♛";
    display: inline-block;
    margin-right: 8px;
    color: hsl(50, 95%, 60%);
}

#block-factory {
    bottom: 16px;
    top: auto;
    text-align: center;
    padding-bottom: 16px;
}
.block-maker {
    display: inline-block;
    padding: 8px 16px;
    background: hsl(250, 10%, 40%);
    margin-right: 8px;
}

#selection-cursor {
    position: absolute;
    display: block;
    width: 32px;
    height: 32px;
    outline: 2px solid hsl(250, 10%, 60%);
}
.selected-block {
    outline: 2px solid hsl(250, 10%, 60%);
}
</style>

<script>
import * as K from './klib.js';

const BLOCK_SIZE = 32;

//============ STATE ============//
/** @typedef {{name: string, block_id: number}} User */
/** @typedef {{type: string}} Block*/
/** @typedef {{owner: number | null, users: {[id: number]: User}, blocks: {[id: number]: Block}, block_owners: {[block_id: number]: number}, positions: {[id: number]: [number, number]} }} City*/

/** @type{City | null} */
let city = null; 
/** @type{number[]} */
let users = [];
$: {
    if (city && city.users) {
        let tmp = Object.keys(city.users).map(Number);
        tmp.sort();
        users = tmp;
    }
}

function update(){
    K.xhr("GET", "/api/" + cityID + "/get").then(function(data) {
        city = JSON.parse(data);
    });
}

function getUserColor(user_id) {
    return ["orange", "green", "purple", "blue", "blue"][user_id % 5];
}

function generateName() {
    let cons = "bcdfghjklmnpqrstvwxyz";
    let vowel = "aeiouy";
    return cons[(Math.random() * cons.length) | 0] + vowel[(Math.random() * vowel.length) | 0] + cons[(Math.random() * cons.length) | 0] + vowel[(Math.random() * vowel.length) | 0];
}

let cityID = location.pathname.split("/")[1] | 0;
let userID;   
let userBlockID;
let userPosition = [0, 0];
let cameraPosition = [0, 0];
$: cameraPosition = userPosition || [0, 0];
$: userPosition = city && city.positions[city.users[userID].block_id];
$: userBlockID = city && city.users[userID].block_id;
K.xhr("POST", "/api/" + cityID + "/join?name=anonymous+" + generateName()).then(function(resp) {
    userID = parseInt(resp);
    update();
    setInterval(update, 1000);
});

    
document.addEventListener("keydown", function(e) {
    let position = userPosition || [0, 0];
    switch (e.key) {
    case "ArrowUp":
        position[1] -= 1; break;
    case "ArrowDown":
        position[1] += 1;  break;
    case "ArrowLeft":
        position[0] -= 1;  break;
    case "ArrowRight":
        position[0] += 1;  break;
    default: return;
    }

    K.xhr("POST", "/api/" + cityID + "/move?user_id=" + userID + "&block_id=" + userBlockID + "&x=" + position[0] + "&y=" + position[1]).then(update);
})

window.addEventListener("beforeunload", function(e) {
    navigator.sendBeacon('/api/' + cityID + "/leave?user_id=" + userID);
});

let screenWidth = window.innerWidth;
let screenHeight = window.innerHeight;

window.addEventListener("resize", () => {
    screenWidth = window.innerWidth;
    screenHeight = window.innerHeight;
});

let selectedBlock = null;
function selectBlock() {
    let id = Number(this.id.slice("block-".length));
    if (city.block_owners[id] == userID || city.owner == userID) {
        selectedBlock = id;
    }
}
function moveSelection() {
    let position = [
        (Number(this.style.left.slice(0, -2)) - screenWidth/2) / BLOCK_SIZE + cameraPosition[0],
        (Number(this.style.top.slice(0, -2)) - screenHeight/2) / BLOCK_SIZE + cameraPosition[1],
    ];
    K.xhr("POST", "/api/" + cityID + "/move?user_id=" + userID + "&block_id=" + selectedBlock + "&x=" + position[0] + "&y=" + position[1]).then(update);
    selectedBlock = null;
}

let mouseX = 0, mouseY = 0;
window.addEventListener("mousemove", function(event) {
    mouseX = event.clientX;
    mouseY = event.clientY;
    console.log(mouseX);
})

function makeObject(type) {
    
}
            </script>

{#if city}
<div id="user-list">
    <h3>Users</h3>
{#each users as user_id}
    {@const user = city.users[user_id]}
    <div class="user user-{getUserColor(user_id)}" class:user-owner={city.owner == user_id}>{user.name}</div>
{/each}
</div>
<div id="blocks">
    {#each Object.keys(city.blocks) as block_id}
    {@const block = city.blocks[block_id]}
    {@const position = city.positions[block_id] || [0,0]}
    {@const owner_id = city.block_owners[block_id]}
    <div class="block block-{block.type == 'User' ? getUserColor(owner_id) : ''}"
        id="block-{block_id}"
        class:block-wall={block.type == "Wall"}
        class:selected-block={block_id == selectedBlock}
        style:color="red"
        style:left="{(BLOCK_SIZE*(position[0]-cameraPosition[0]) + screenWidth / 2)}px"
        style:top="{(BLOCK_SIZE*(position[1]-cameraPosition[1]) + screenHeight / 2)}px"
        on:click={selectBlock}></div>
{/each}

{#if selectedBlock != null}
<div id="selection-cursor"
    style:left="{Math.floor((mouseX-screenWidth/2)/BLOCK_SIZE)*BLOCK_SIZE + screenWidth/2}px"
    style:top="{Math.floor((mouseY-screenHeight/2)/BLOCK_SIZE)*BLOCK_SIZE + screenHeight/2}px"
    on:click={moveSelection}></div>
{/if}
</div>

<div id="block-factory">
    <h3>Factory</h3>
    <div class="block-maker" on:click={() => makeObject("Wall")}>Wall</div>
    <div class="block-maker" on:click={() => makeObject("Video")}>TV</div>
</div>
{/if}