import { Socket } from 'phoenix';


let socket = new Socket("/socket")

socket.connect()


let channel = socket.channel("room:1", {})

channel.join()
    .receive("ok", resp => { console.log("Joined successfully", resp) })
    .receive("error", resp => { console.log("Unable to join", resp) })

channel.push("test", { data: "join room:1" })
    .receive("ok", resp => { console.log("Pushed successfully", resp) })
    .receive("error", resp => { console.log("Unable to push", resp) })


// let channel2 = socket.channel("room:2", {})

// channel2.join()
//     .receive("ok", resp => { console.log("Joined successfully", resp) })
//     .receive("error", resp => { console.log("Unable to join", resp) })

// channel2.push("test", { data: "join room:2" })
//     .receive("ok", resp => { console.log("Pushed successfully", resp) })
//     .receive("error", resp => { console.log("Unable to push", resp) })


// channel.leave()
