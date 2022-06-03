import { Socket } from './phoenix';


let socket = new Socket("/socket")

socket.connect()


let channel = socket.channel("room:lobby", {})

channel.join()
    .receive("ok", resp => { console.log("Joined successfully", resp) })
    .receive("error", resp => { console.log("Unable to join", resp) })


channel.push("test", { data: "test123" })

channel.on("boardcast", resp => {
    console.log("boardcast", resp)
})

// channel.leave()
