package com.reflex.remote.data

import com.reflex.remote.data.model.CommandCategory
import com.reflex.remote.data.model.ReflexCommandItem

typealias CommandCategory = com.reflex.remote.data.model.CommandCategory
typealias ReflexCommandItem = com.reflex.remote.data.model.ReflexCommandItem

object CommandDirectoryData {
    // Commands are now loaded dynamically and modularly from JSON assets in assets/commands/
    val items: List<ReflexCommandItem> = emptyList()
}
