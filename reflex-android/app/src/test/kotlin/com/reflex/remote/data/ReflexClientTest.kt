package com.reflex.remote.data

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ReflexClientTest {

    @Test
    fun testReflexResponseSuccessParsing() {
        val resp = ReflexResponse(
            status = "Success",
            action = "VolumeSet",
            durationUs = 120L,
            roundTripMs = 5L
        )

        assertTrue(resp.isSuccess)
        assertEquals("VolumeSet", resp.action)
        assertEquals(120L, resp.durationUs)
    }

    @Test
    fun testReflexResponseErrorParsing() {
        val resp = ReflexResponse(
            status = "Error",
            reason = "No selected file detected",
            action = "FileActionFailed",
            durationUs = 90L,
            roundTripMs = 4L
        )

        assertFalse(resp.isSuccess)
        assertEquals("No selected file detected", resp.reason)
    }

    @Test
    fun testReflexResponsePassThroughParsing() {
        val resp = ReflexResponse(
            status = "PassThrough",
            reason = "NoGrammarMatch",
            durationUs = 35L
        )

        assertFalse(resp.isSuccess)
        assertEquals("NoGrammarMatch", resp.reason)
    }
}
