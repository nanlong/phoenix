import { Socket } from './phoenix';


let socket = new Socket("/socket")

socket.connect()


let channel = socket.channel("room:lobby", {})

channel.join()
    .receive("ok", resp => { console.log("Joined successfully", resp) })
    .receive("error", resp => { console.log("Unable to join", resp) })


let channel2 = socket.channel("room:lobby", {})

channel2.join()
    .receive("ok", resp => { console.log("Joined successfully", resp) })
    .receive("error", resp => { console.log("Unable to join", resp) })


// channel.on("boardcast", resp => {
//     console.log("boardcast", resp)
// })

// channel.push("test", { data: "test123" })
// // setInterval(() => {
// //     channel.push("test", { data: "test123" })
// // }, 10)


// channel.leave()
