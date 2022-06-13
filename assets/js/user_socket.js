import { Socket } from 'phoenix';


let socket = new Socket("/socket")

socket.connect()


let channel = socket.channel("room:*", {})

channel.join()
    .receive("ok", resp => { console.log("Joined successfully", resp) })
    .receive("error", resp => { console.log("Unable to join", resp) })

channel.push("test", { data: "test123" })
    .receive("ok", resp => { console.log("Pushed successfully", resp) })
    .receive("error", resp => { console.log("Unable to push", resp) })

channel.on("boardcast", resp => {
    console.log("receive boardcast message:", resp);
})

// channel.leave()
